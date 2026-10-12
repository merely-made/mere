// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

// Prepared OS AX probe. Root compiles/runs under the serialized native handoff.
// It never launches an application, prompts for permission, or changes settings.
import Foundation
import AppKit
import ApplicationServices

struct Refusal: Error { let reason: String }
let started = Date()
let deadline: TimeInterval = 20
let nodeLimit = 2048
func checkDeadline() throws {
    if Date().timeIntervalSince(started) > deadline { throw Refusal(reason: "AX probe deadline") }
}
var options: [String: String] = [:]
var trustOnly = false
var argv = Array(CommandLine.arguments.dropFirst())
while !argv.isEmpty {
    let key = argv.removeFirst()
    if key == "--trust-only" { trustOnly = true; continue }
    guard key.hasPrefix("--"), !argv.isEmpty else {
        fputs("Expected --trust-only or --pid/--executable/--output/--action VALUE\n", stderr)
        exit(64)
    }
    options[key] = argv.removeFirst()
}
var receipt: [String: Any] = [
    "scope": "actual macOS AX server tree and actions for one explicitly owned PID",
    "executed": true, "harness": false, "permission_prompt": false,
    "timeout_seconds": deadline, "node_limit": nodeLimit,
    "trusted": AXIsProcessTrusted(), "driver_reset": false
]
func finish(_ status: String, _ code: Int32) -> Never {
    receipt["status"] = status
    receipt["elapsed_seconds"] = Date().timeIntervalSince(started)
    do {
        let bytes = try JSONSerialization.data(withJSONObject: receipt, options: [.prettyPrinted, .sortedKeys])
        if let path = options["--output"] {
            let url = URL(fileURLWithPath: path)
            guard !FileManager.default.fileExists(atPath: path) else { throw Refusal(reason: "receipt already exists") }
            try bytes.write(to: url, options: .withoutOverwriting)
        }
        FileHandle.standardOutput.write(bytes)
        print()
    } catch { fputs("Cannot write AX receipt: \(error)\n", stderr); exit(74) }
    exit(code)
}
guard AXIsProcessTrusted() else { finish("permission_unavailable_no_prompt_no_actions", 2) }
if trustOnly { finish("permission_available_for_this_probe_identity", 0) }
guard let pidString = options["--pid"], let pid = Int32(pidString), pid > 1,
      let executable = options["--executable"], let application = NSRunningApplication(processIdentifier: pid),
      let actualExecutable = application.executableURL else { finish("explicit_owned_pid_and_executable_required", 64) }
guard actualExecutable.resolvingSymlinksInPath().path == URL(fileURLWithPath: executable).resolvingSymlinksInPath().path else {
    finish("target_executable_identity_mismatch_no_actions", 3)
}
receipt["pid"] = pid
receipt["executable"] = actualExecutable.path
let app = AXUIElementCreateApplication(pid)
AXUIElementSetMessagingTimeout(app, 0.25)
func attribute(_ element: AXUIElement, _ name: String) throws -> Any? {
    try checkDeadline()
    var value: CFTypeRef?
    let error = AXUIElementCopyAttributeValue(element, name as CFString, &value)
    if error == .success { return value }
    if error == .attributeUnsupported || error == .noValue { return nil }
    throw Refusal(reason: "\(name) AX error \(error.rawValue)")
}
func label(_ element: AXUIElement) throws -> String {
    for name in ["AXTitle", "AXDescription", "AXHelp"] {
        if let text = try attribute(element, name) as? String, !text.isEmpty { return text }
    }
    return ""
}
struct Entry { let element: AXUIElement; let parent: Int?; let depth: Int; let label: String; let role: String }
func tree() throws -> ([Entry], [[String: Any]]) {
    var entries: [Entry] = []
    var records: [[String: Any]] = []
    var queue: [(AXUIElement, Int?, Int)] = [(app, nil, 0)]
    var cursor = 0
    while cursor < queue.count {
        try checkDeadline()
        let (element, parent, depth) = queue[cursor]; cursor += 1
        if entries.contains(where: { CFEqual($0.element, element) }) { continue }
        guard entries.count < nodeLimit, depth <= 32 else { throw Refusal(reason: "AX tree quota") }
        AXUIElementSetMessagingTimeout(element, 0.25)
        let role = try attribute(element, "AXRole") as? String ?? ""
        let title = try label(element)
        let index = entries.count
        entries.append(Entry(element: element, parent: parent, depth: depth, label: title, role: role))
        var record: [String: Any] = ["id": index, "depth": depth, "role": role, "label": title]
        if let parent { record["parent"] = parent }
        for name in ["AXIdentifier", "AXValue", "AXFocused", "AXEnabled", "AXMinValue", "AXMaxValue"] {
            if let value = try attribute(element, name), value is String || value is NSNumber { record[name] = value }
        }
        var actions: CFArray?
        try checkDeadline()
        if AXUIElementCopyActionNames(element, &actions) == .success { record["actions"] = actions as? [String] ?? [] }
        var settable: DarwinBoolean = false
        try checkDeadline()
        let writable = AXUIElementIsAttributeSettable(element, "AXValue" as CFString, &settable)
        record["value_settable"] = writable == .success && settable.boolValue
        records.append(record)
        var count: CFIndex = 0
        let childrenName = depth == 0 ? "AXWindows" : "AXChildren"
        try checkDeadline()
        let counted = AXUIElementGetAttributeValueCount(element, childrenName as CFString, &count)
        if counted == .attributeUnsupported || counted == .noValue { continue }
        guard counted == .success else { throw Refusal(reason: "children count AX error \(counted.rawValue)") }
        guard count <= nodeLimit - queue.count else { throw Refusal(reason: "AX child quota") }
        if count > 0 {
            var children: CFArray?
            try checkDeadline()
            let copied = AXUIElementCopyAttributeValues(element, childrenName as CFString, 0, count, &children)
            guard copied == .success else { throw Refusal(reason: "children AX error \(copied.rawValue)") }
            for child in children as? [AXUIElement] ?? [] { queue.append((child, index, depth + 1)) }
        }
    }
    return (entries, records)
}
do {
    let (entries, records) = try tree()
    receipt["before"] = records
    let action = options["--action"] ?? "inspect"
    receipt["action"] = action
    if action == "inspect" { finish("os_tree_observed_actions_not_executed", 0) }
    func inPreview(_ entry: Entry) -> Bool {
        var parent = entry.parent
        while let index = parent {
            if entries[index].label == "Preview mode" { return true }
            parent = entries[index].parent
        }
        return false
    }
    let candidates = entries.filter { entry in
        switch action {
        case "mode-press": return entry.role == "AXButton" && entry.label == "Dark" && inPreview(entry)
        case "hue-set": return entry.role == "AXSlider" && entry.label == "Primary hue"
        case "focus-name", "text-set": return entry.role == "AXTextField" && entry.label == "Theme name"
        default: return false
        }
    }
    guard candidates.count == 1 else { throw Refusal(reason: "unique supported OS target required; matches=\(candidates.count)") }
    let target = candidates[0].element
    let previousValue = try attribute(target, "AXValue")
    receipt["target_role"] = candidates[0].role
    receipt["target_label"] = candidates[0].label
    if let previousValue, previousValue is String || previousValue is NSNumber { receipt["previous_value"] = previousValue }
    var result: AXError
    switch action {
    case "mode-press": result = AXUIElementPerformAction(target, "AXPress" as CFString)
    case "focus-name": result = AXUIElementSetAttributeValue(target, "AXFocused" as CFString, kCFBooleanTrue)
    case "hue-set", "text-set":
        var settable: DarwinBoolean = false
        guard AXUIElementIsAttributeSettable(target, "AXValue" as CFString, &settable) == .success && settable.boolValue else {
            finish("requested_OS_value_action_not_supported", 4)
        }
        let value: CFTypeRef = action == "hue-set" ? NSNumber(value: 0.37) : "Release-AX-draft" as CFString
        result = AXUIElementSetAttributeValue(target, "AXValue" as CFString, value)
    default: throw Refusal(reason: "unknown action")
    }
    receipt["action_AX_error"] = result.rawValue
    guard result == .success else { throw Refusal(reason: "OS action refused") }
    let observeUntil = Date().addingTimeInterval(3)
    repeat {
        try checkDeadline()
        Thread.sleep(forTimeInterval: 0.1)
        if action == "hue-set", let number = try attribute(target, "AXValue") as? NSNumber,
           abs(number.doubleValue - 0.37) < 0.0001 { receipt["numeric_value_observed"] = true; break }
        if action == "text-set", let text = try attribute(target, "AXValue") as? String,
           text == "Release-AX-draft" { receipt["text_value_observed"] = true; break }
        if action == "focus-name", let focused = try attribute(target, "AXFocused") as? NSNumber,
           focused.boolValue { receipt["focus_observed"] = true; break }
    } while Date() < observeUntil
    let (_, after) = try tree()
    receipt["after"] = after
    if action == "hue-set" && receipt["numeric_value_observed"] == nil { throw Refusal(reason: "numeric setter had no OS value effect") }
    if action == "text-set" && receipt["text_value_observed"] == nil { throw Refusal(reason: "text setter had no OS value effect") }
    if action == "focus-name" && receipt["focus_observed"] == nil { throw Refusal(reason: "focus setter had no OS focus effect") }
    finish("OS_action_delivered_application_receipt_still_required", 0)
} catch { receipt["refusal"] = String(describing: error); finish("refused_or_unavailable", 5) }
