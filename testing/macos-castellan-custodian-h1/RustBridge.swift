// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
import Foundation
import Security
import AppKit
import LocalAuthentication
import Darwin

// Linked only into CUSTODIAN. The expected path is compiled into the binary;
// a copied bundle or changed resources cannot choose another privileged root.
func validateCustodianInstallation() throws {
    guard Bundle.main.bundleURL.standardizedFileURL.path == expectedCustodianBundlePath else {
        throw NSError(domain:"CustodianInstallation",code:1)
    }
    var url = Bundle.main.bundleURL
    while url.path != "/" {
        var attributes = stat()
        guard lstat(url.path, &attributes) == 0, attributes.st_uid == 0,
              attributes.st_mode & 0o022 == 0, attributes.st_mode & S_IFMT != S_IFLNK else {
            throw NSError(domain:"CustodianInstallation",code:2)
        }
        url.deleteLastPathComponent()
    }
    var code: SecStaticCode?
    guard SecStaticCodeCreateWithPath(Bundle.main.bundleURL as CFURL, [], &code) == errSecSuccess,
          let code, SecStaticCodeCheckValidity(code, SecCSFlags(rawValue:kSecCSStrictValidate), nil) == errSecSuccess else {
        throw NSError(domain:"CustodianInstallation",code:3)
    }
}

enum RustCustody {
    static let capacity = 64 * 1024
    static func result(_ invoke: (UnsafeMutablePointer<UInt8>, Int)->Int) -> [String:Any] {
        var bytes=[UInt8](repeating:0,count:capacity)
        let count=bytes.withUnsafeMutableBufferPointer { invoke($0.baseAddress!,capacity) }
        guard count>0 && count<=capacity,
              let reply=try? JSONSerialization.jsonObject(with:Data(bytes.prefix(count))) as? [String:Any] else {
            return ["refused":"invalid-native-response"]
        }
        return reply
    }
    static func initialize(_ c:[String:Any], newlyCreatedKey:Bool) throws -> [String:Any] {
        let value:[String:Any] = ["run_id":c["runID"]!,"directory":c["castellanDirectory"]!,
                                 "profile":"h1-synthetic","principal":c["controllerID"]!]
        let config=try JSONSerialization.data(withJSONObject:value)
        let (status,key)=copyKey(c)
        guard status==errSecSuccess, var key, key.count==32 else {throw NSError(domain:"CustodianKey",code:Int(status))}
        defer {key.resetBytes(in:0..<key.count)}
        return config.withUnsafeBytes { configBytes in key.withUnsafeBytes { keyBytes in
            result { djinn_custodian_initialize(configBytes.bindMemory(to:UInt8.self).baseAddress!,config.count,
                keyBytes.bindMemory(to:UInt8.self).baseAddress!,key.count,newlyCreatedKey,$0,$1) }
        }}
    }
    static func dispatch(_ value:[String:Any], principal:String) -> [String:Any] {
        guard let request=try? JSONSerialization.data(withJSONObject:value),request.count<=16*1024 else {return ["refused":"request-too-large"]}
        let who=Data(principal.utf8)
        return who.withUnsafeBytes { p in request.withUnsafeBytes { r in
            result { djinn_custodian_dispatch(p.bindMemory(to:UInt8.self).baseAddress!,who.count,
                r.bindMemory(to:UInt8.self).baseAddress!,request.count,$0,$1) }
        }}
    }
    static func status(_ c:[String:Any]) -> [String:Any] {
        dispatch(["op":"status"],principal:c["controllerID"] as! String)
    }
    static func confirm(_ ticket:String, accepted:Bool) -> [String:Any] {
        let bytes=Data(ticket.utf8)
        return bytes.withUnsafeBytes { p in result { djinn_custodian_confirm(p.bindMemory(to:UInt8.self).baseAddress!,bytes.count,accepted,$0,$1) } }
    }
    static func ask(_ facts:[String:Any], reply:@escaping (NSDictionary)->Void) {
        // The facts/ticket come from Rust's pending request, not client fields.
        DispatchQueue.main.async {
            let app=NSApplication.shared;app.setActivationPolicy(.accessory);app.activate(ignoringOtherApps:true)
            let alert=NSAlert();alert.messageText="Djinn H1 · Synthetic Castellan confirmation"
            alert.informativeText="Operation: \(facts["operation"] ?? "unknown")\nPersona: \(facts["profile"] ?? "unknown")\nRequester: \(facts["principal"] ?? "unknown")\nScope or payload digest: \(facts["payload_digest"] ?? "unknown")\n\nApprove this synthetic test operation?"
            alert.addButton(withTitle:"Deny");alert.addButton(withTitle:"Approve")
            let accepted=alert.runModal()==NSApplication.ModalResponse.alertSecondButtonReturn
            guard let ticket=facts["ticket"] as? String else {reply(["refused":"native-confirmation-facts"]);return}
            reply(confirm(ticket,accepted:accepted) as NSDictionary)
        }
    }
    static func unlock(_ c:[String:Any], reply:@escaping (NSDictionary)->Void) {
        let context=LAContext();var error:NSError?
        guard context.canEvaluatePolicy(.deviceOwnerAuthentication,error:&error) else {reply(["refused":"native-presence-unavailable"]);return}
        context.evaluatePolicy(.deviceOwnerAuthentication,localizedReason:"Unlock the synthetic Castellan receiving vault") { [context] success,_ in
            defer {context.invalidate()}
            guard success else {reply(["refused":"native-unlock-denied"]);return}
            let (status,key)=copyKey(c)
            guard status==errSecSuccess,var key,key.count==32 else {reply(["refused":"native-unlock-key"]);return}
            defer {key.resetBytes(in:0..<key.count)}
            let response=key.withUnsafeBytes { p in result { djinn_custodian_unlock(p.bindMemory(to:UInt8.self).baseAddress!,key.count,$0,$1) } }
            reply(response as NSDictionary)
        }
    }
}
