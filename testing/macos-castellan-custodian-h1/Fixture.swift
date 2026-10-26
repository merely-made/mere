// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
// Synthetic macOS enforcement fixture. Never imports a production vault or profile.
import Foundation
import Security
import CryptoKit
import ServiceManagement
import LocalAuthentication
import Darwin

@objc protocol H1Protocol {
    func request(_ value: NSDictionary, reply: @escaping (NSDictionary) -> Void)
}

func emit(_ value: [String: Any]) {
    if let data = try? JSONSerialization.data(withJSONObject: value, options: [.sortedKeys]),
       let string = String(data: data, encoding: .utf8) { print(string); fflush(stdout) }
}
func digest(_ data: Data) -> String { SHA256.hash(data: data).map { String(format: "%02x", $0) }.joined() }
func config() throws -> [String: Any] {
    let url = Bundle.main.url(forResource: "fixture", withExtension: "json")!
    return try JSONSerialization.jsonObject(with: Data(contentsOf: url)) as! [String: Any]
}
func requirement(_ id: String, team: String) -> String {
    return "anchor apple generic and certificate leaf[subject.OU] = \"\(team)\" and identifier \"\(id)\""
}
func keyQuery(_ c: [String: Any]) -> [String: Any] {
    let context = LAContext(); context.interactionNotAllowed = true
    return [kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: c["keyService"]!, kSecAttrAccount as String: c["runID"]!,
            kSecAttrAccessGroup as String: c["keyGroup"]!,
            kSecUseDataProtectionKeychain as String: true,
            kSecUseAuthenticationContext as String: context,
            kSecAttrSynchronizable as String: false]
}
func copyKey(_ c: [String: Any]) -> (OSStatus, Data?) {
    var q = keyQuery(c); q[kSecReturnData as String] = true
    var result: CFTypeRef?
    let status = SecItemCopyMatching(q as CFDictionary, &result)
    return (status, result as? Data)
}
func keyPreflight(_ c: [String: Any]) -> [String: Any] {
    // Creation and retrieval use only this fresh run's exact synthetic item.
    let (found, old) = copyKey(c)
    if found == errSecSuccess { return ["keyStatus": found, "keyBytes": old?.count ?? 0, "existing": true] }
    guard found == errSecItemNotFound else { return ["keyStatus": found, "stage": "query"] }
    var random = Data(count: 32)
    let randomStatus = random.withUnsafeMutableBytes { SecRandomCopyBytes(kSecRandomDefault, 32, $0.baseAddress!) }
    guard randomStatus == errSecSuccess else { return ["keyStatus": randomStatus, "stage": "random"] }
    var q = keyQuery(c)
    q[kSecValueData as String] = random
    q[kSecAttrAccessible as String] = kSecAttrAccessibleWhenUnlockedThisDeviceOnly
    let status = SecItemAdd(q as CFDictionary, nil)
    random.resetBytes(in: 0..<random.count)
    if status != errSecSuccess && status != errSecDuplicateItem { return ["keyStatus": status, "stage": "add"] }
    let (read, key) = copyKey(c)
    return ["keyStatus": read, "keyBytes": key?.count ?? 0, "existing": status != errSecSuccess]
}
func tryRead(_ path: String) -> [String: Any] {
    let fd = open(path, O_RDONLY | O_CLOEXEC)
    if fd < 0 { return ["opened": false, "errno": errno] }
    defer { close(fd) }
    var byte: UInt8 = 0; let result = read(fd, &byte, 1)
    return ["opened": true, "readBytes": result, "errno": result < 0 ? errno : 0]
}
func readAccessProbe(_ path: String) -> [String: Any] {
    // Test access to the signing account's Keychain file without reading bytes.
    let fd = open(path, O_RDONLY | O_NOFOLLOW | O_CLOEXEC)
    if fd < 0 { return ["opened": false, "errno": errno] }
    close(fd); return ["opened": true]
}
func writeAccessProbe(_ path: String) -> [String: Any] {
    // Opening for append tests write authority without changing even a byte.
    let fd = open(path, O_WRONLY | O_APPEND | O_NOFOLLOW | O_CLOEXEC)
    if fd < 0 { return ["writable": false, "errno": errno] }
    close(fd); return ["writable": true]
}
func directoryCreateProbe(_ directory: String) -> [String: Any] {
    let path = directory + "/.h1-write-probe-" + UUID().uuidString
    let fd = open(path, O_WRONLY | O_CREAT | O_EXCL | O_NOFOLLOW | O_CLOEXEC, 0o600)
    if fd < 0 { return ["created": false, "errno": errno] }
    close(fd); unlink(path); return ["created": true]
}
func networkProbe(_ port: UInt16) -> [String: Any] {
    let fd = socket(AF_INET, SOCK_STREAM, 0)
    if fd < 0 { return ["connected": false, "stage": "socket", "errno": errno] }
    defer { close(fd) }
    var address = sockaddr_in(); address.sin_len = UInt8(MemoryLayout<sockaddr_in>.size)
    address.sin_family = sa_family_t(AF_INET); address.sin_port = port.bigEndian
    address.sin_addr.s_addr = inet_addr("127.0.0.1")
    let result = withUnsafePointer(to: &address) { p in
        p.withMemoryRebound(to: sockaddr.self, capacity: 1) { connect(fd, $0, socklen_t(MemoryLayout<sockaddr_in>.size)) }
    }
    return ["connected": result == 0, "errno": result < 0 ? errno : 0]
}
func memoryProbe(_ pid: Int32) -> [String: Any] {
    var task: mach_port_t = 0
    let status = task_for_pid(mach_task_self_, pid, &task)
    if status == KERN_SUCCESS { mach_port_deallocate(mach_task_self_, task) }
    return ["taskPortObtained": status == KERN_SUCCESS, "kernReturn": status]
}
func workerProbe(_ c: [String: Any]) -> [String: Any] {
    var inherited: [String] = []
    let vaultRoot = (c["vaultCanary"] as! String).components(separatedBy: "/custody/")[0]
    var inheritedCanary = false
    for fd in 0..<getdtablesize() {
        var status = stat()
        if fstat(fd, &status) == 0, (status.st_mode & S_IFMT) == S_IFREG {
            var bytes = [UInt8](repeating: 0, count: 32)
            let count = pread(fd, &bytes, 32, 0)
            if count == 32, let expected = c["vaultCanaryHash"] as? String,
               digest(Data(bytes)) == expected { inheritedCanary = true }
        }
        var name = [CChar](repeating: 0, count: Int(MAXPATHLEN))
        if fcntl(Int32(fd), F_GETPATH, &name) == 0 {
            let path = String(cString: name)
            if path.hasPrefix(vaultRoot) { inherited.append(path) }
        }
    }
    let selected = c["selectedInput"] as? String ?? ""
    let key = copyKey(c)
    var result: [String: Any] = ["pid": getpid(), "ppid": getppid(), "uid": getuid(),
        "vault": tryRead(c["vaultCanary"] as! String),
        "symlink": tryRead(c["vaultSymlink"] as! String),
        "blob": tryRead(c["blobCanary"] as! String),
        "inheritedFixturePaths": inherited, "inheritedVaultCanary": inheritedCanary,
        "selfMemoryControl": memoryProbe(getpid()),
        "selectedInputHash": digest(Data(selected.utf8)),
        "keyStatus": key.0, "keyObtained": key.0 == errSecSuccess]
    if key.0 == errSecSuccess, let bytes = key.1,
       let sealedPath = c["sealedVault"] as? String,
       let sealedBytes = try? Data(contentsOf: URL(fileURLWithPath: sealedPath)),
       let sealed = try? AES.GCM.SealedBox(combined: sealedBytes),
       var seed = try? AES.GCM.open(sealed, using: SymmetricKey(data: bytes),
                                   authenticating: Data((c["runID"] as! String).utf8)) {
        defer { seed.resetBytes(in: 0..<seed.count) }
        if let root = try? Curve25519.Signing.PrivateKey(rawRepresentation: seed) {
            result["syntheticRootPublicKey"] = root.publicKey.rawRepresentation.base64EncodedString()
        }
    }
    if let targets = c["protectedTargets"] as? [String: String] {
        result["protectedWrites"] = targets.mapValues { writeAccessProbe($0) }
    }
    if let directory = c["protectedDirectory"] as? String { result["protectedCreate"] = directoryCreateProbe(directory) }
    if let keychain = c["signerKeychainFile"] as? String { result["signerKeychainFile"] = readAccessProbe(keychain) }
    if let port = c["networkPort"] as? Int { result["network"] = networkProbe(UInt16(port)) }
    if let pid = c["custodianPID"] as? Int { result["memory"] = memoryProbe(Int32(pid)) }
    // Own-container write/read is the positive file access control.
    let own = FileManager.default.temporaryDirectory.appendingPathComponent("h1-own-\(UUID().uuidString)")
    do {
        try Data("synthetic own-container control".utf8).write(to: own)
        result["ownFileControl"] = try Data(contentsOf: own) == Data("synthetic own-container control".utf8)
        try FileManager.default.removeItem(at: own)
    } catch { result["ownFileControl"] = false }
    return result
}

// Each native connection pins code identity before resume; a public exchange
// must succeed before any secret-bearing operation is transmitted.
func rpc(_ connection: NSXPCConnection, _ value: [String: Any], timeout: TimeInterval = 4) -> [String: Any] {
    connection.remoteObjectInterface = NSXPCInterface(with: H1Protocol.self)
    let lock = NSLock(); var answer: [String: Any]?
    let proxy = connection.remoteObjectProxyWithErrorHandler { error in
        lock.lock(); defer { lock.unlock() }
        answer = ["transportError": (error as NSError).code]
    } as! H1Protocol
    proxy.request(value as NSDictionary) { result in
        lock.lock(); defer { lock.unlock() }; answer = result as? [String: Any]
    }
    let deadline = Date().addingTimeInterval(timeout)
    while Date() < deadline {
        lock.lock(); let current = answer; lock.unlock()
        if let current { return current }
        RunLoop.current.run(until: Date().addingTimeInterval(0.02))
    }
    return ["timeout": true]
}
func custodyConnection(_ c: [String: Any], service: String? = nil) -> NSXPCConnection {
    let conn = NSXPCConnection(machServiceName: service ?? c["machService"] as! String)
    conn.setCodeSigningRequirement(requirement(c["custodianID"] as! String, team: c["team"] as! String))
    conn.resume(); return conn
}

#if CUSTODIAN
final class Custodian: NSObject, H1Protocol, NSXPCListenerDelegate {
    let c: [String: Any]
    let serial = NSLock()
    var connections: [NSXPCConnection] = []
    init(_ c: [String: Any]) { self.c = c }
    func listener(_ listener: NSXPCListener, shouldAcceptNewConnection conn: NSXPCConnection) -> Bool {
        serial.lock(); defer { serial.unlock() }
        guard connections.count < 8 else { return false }
        conn.exportedInterface = NSXPCInterface(with: H1Protocol.self); conn.exportedObject = self
        conn.invalidationHandler = { [weak self, weak conn] in
            guard let self, let conn else { return }
            self.serial.lock(); self.connections.removeAll { $0 === conn }; self.serial.unlock()
        }
        connections.append(conn); conn.resume(); return true
    }
    func request(_ value: NSDictionary, reply: @escaping (NSDictionary) -> Void) {
        guard let value=value as? [String:Any] else {reply(["refused":"invalid-request"]);return}
        let op=value["op"] as? String ?? ""
        if op=="hello" {
            let state=RustCustody.status(c)
            reply(["ok":state["ok"] ?? false,"instance":state["instance"] ?? "", "pid":getpid(),"ppid":getppid(),
                "uid":getuid(),"runID":c["runID"]!,"key":keyPreflight(c),"castellan":state]);return
        }
        let state=RustCustody.status(c)
        guard value["instance"] as? String==state["instance"] as? String else {reply(["refused":"instance"]);return}
        if op=="fixture-delete-key" {reply(["deleteStatus":SecItemDelete(keyQuery(c) as CFDictionary)]);return}
        if op=="native-unlock" {
            guard Set(value.keys)==Set(["op","instance"]) else {reply(["refused":"invalid-request"]);return}
            RustCustody.unlock(c,reply:reply);return
        }
        let response=RustCustody.dispatch(value,principal:c["controllerID"] as! String)
        if response["nativeConfirmationRequired"] as? Bool==true,let facts=response["facts"] as? [String:Any] {
            RustCustody.ask(facts,reply:reply)
        } else {reply(response as NSDictionary)}
    }

}
#endif

#if WORKER
final class Worker: NSObject, H1Protocol, NSXPCListenerDelegate {
    var connections: [NSXPCConnection] = []
    func listener(_ listener: NSXPCListener, shouldAcceptNewConnection conn: NSXPCConnection) -> Bool {
        conn.setCodeSigningRequirement(requirement(Bundle.main.object(forInfoDictionaryKey: "H1ControllerID") as! String,
                                                   team: Bundle.main.object(forInfoDictionaryKey: "H1Team") as! String))
        conn.exportedInterface = NSXPCInterface(with: H1Protocol.self); conn.exportedObject = self
        connections.append(conn); conn.resume(); return true
    }
    func request(_ value: NSDictionary, reply: @escaping (NSDictionary) -> Void) {
        guard let c = value as? [String: Any], (c["selectedInput"] as? String)?.utf8.count ?? 0 <= 4096 else {
            reply(["refused": "invalid-fixture"]); return
        }
        reply(workerProbe(c) as NSDictionary)
    }
}
#endif

#if OUTSIDER
final class FakeCustodian: NSObject, H1Protocol, NSXPCListenerDelegate {
    func listener(_ listener: NSXPCListener, shouldAcceptNewConnection conn: NSXPCConnection) -> Bool {
        conn.exportedInterface = NSXPCInterface(with: H1Protocol.self); conn.exportedObject = self
        conn.resume(); return true
    }
    func request(_ value: NSDictionary, reply: @escaping (NSDictionary) -> Void) {
        let c = try! config()
        emit(["fakeReceivedOp": value["op"] ?? "missing", "secretPresent": value["challenge"] != nil,
              "lane": value["lane"] ?? "missing"])
        reply(["ok": true, "instance": "fake", "runID": c["runID"]!])
    }
}
#endif

#if CONTROLLER
func castellanE2E(_ c:[String:Any]) -> [String:Any] {
    func call(_ value:[String:Any], _ timeout:TimeInterval=8) -> [String:Any] {
        let connection=custodyConnection(c);defer {connection.invalidate()}
        return rpc(connection,value,timeout:timeout)
    }
    func verify(_ response:[String:Any], _ message:String) -> Bool {
        guard let p=response["publicKey"] as? String,let bytes=Data(base64Encoded:p),
              let s=response["signature"] as? String,let signature=Data(base64Encoded:s),
              let publicKey=try? Curve25519.Signing.PublicKey(rawRepresentation:bytes) else {return false}
        return publicKey.isValidSignature(signature,for:Data(message.utf8))
    }
    var result:[String:Any]=[:]
    let first=call(["op":"hello"]);result["initialHello"]=first
    guard let oldInstance=first["instance"] as? String,let initial=first["castellan"] as? [String:Any],
          let rootPublic=initial["publicKey"] as? String else {return result}
    result["forgedApproval"]=call(["op":"sign","namespace":"h1.root-sign.v1","challenge":"synthetic forged approval","instance":oldInstance,"approved":true])
    result["clientDecision"]=call(["op":"approve","ticket":"not-a-native-ticket","instance":oldInstance,"accepted":true])
    func open(_ instance:String,_ uses:Int) -> [String:Any] {
        call(["op":"open-session","namespace":"h1.continuous-sign.v1","profile":"h1-synthetic","max_uses":uses,"instance":instance],120)
    }
    func session(_ instance:String,_ grant:[String:Any],namespace:String="h1.continuous-sign.v1",profile:String="h1-synthetic") -> [String:Any] {
        call(["op":"session-sign","instance":instance,"generation":grant["generation"] ?? -1,
              "capability":grant["capability"] ?? "missing","namespace":namespace,"profile":profile,"challenge":"synthetic scoped payload"])
    }
    let firstGrant=open(oldInstance,3);result["firstGrant"]=firstGrant
    let firstSignature=session(oldInstance,firstGrant);result["firstSignature"]=firstSignature
    result["firstSignatureVerified"]=verify(firstSignature,"synthetic scoped payload")
    result["derivedKeyDiffersFromRoot"]=(firstSignature["publicKey"] as? String).map {$0 != rootPublic} ?? false
    result["wrongNamespace"]=session(oldInstance,firstGrant,namespace:"other-domain")
    result["wrongProfile"]=session(oldInstance,firstGrant,profile:"other-persona")
    // Force a real marker rename failure without changing any secret record.
    // The path is the signed synthetic configuration's marker, not caller input.
    let marker=URL(fileURLWithPath:c["castellanDirectory"] as! String).appendingPathComponent("locked")
    do {
        try FileManager.default.createDirectory(at:marker,withIntermediateDirectories:false)
        result["lockFaultInjected"]=true
        result["faultLock"]=call(["op":"lock","instance":oldInstance])
        result["faultLockedHello"]=call(["op":"hello"])
        result["faultRetry"]=call(["op":"lock","instance":oldInstance])
        guard try FileManager.default.contentsOfDirectory(atPath:marker.path).isEmpty else {return result}
        try FileManager.default.removeItem(at:marker)
        result["lockFaultRepaired"]=true
    } catch {result["lockFaultError"]=(error as NSError).code;return result}
    result["lock"]=call(["op":"lock","instance":oldInstance])
    // Model an unrelated same-user app deleting the advisory lock marker.
    do {
        try FileManager.default.removeItem(atPath:(c["castellanDirectory"] as! String)+"/locked")
        result["lockMarkerRemoved"]=true
    } catch {result["lockMarkerRemoved"]=false;return result}
    let restart=Process();restart.executableURL=URL(fileURLWithPath:"/bin/launchctl")
    restart.arguments=["kickstart","-k","gui/\(getuid())/\(c["machService"] as! String)"]
    do {try restart.run();restart.waitUntilExit();result["restartExit"]=restart.terminationStatus} catch {result["restartExit"] = -1}
    let locked=call(["op":"hello"]);result["lockedHello"]=locked
    guard let newInstance=locked["instance"] as? String else {return result}
    result["lockedSign"]=call(["op":"sign","namespace":"h1.root-sign.v1","challenge":"must remain locked","instance":newInstance])
    result["nativeUnlock"]=call(["op":"native-unlock","instance":newInstance],120)
    let unlocked=call(["op":"hello"]);result["unlockedHello"]=unlocked
    result["sameRootAfterNativeUnlock"]=(unlocked["castellan"] as? [String:Any])?["publicKey"] as? String == rootPublic
    result["oldCapabilityAfterUnlock"]=session(newInstance,firstGrant)
    let secondGrant=open(newInstance,3);result["secondGrant"]=secondGrant
    let secondSignature=session(newInstance,secondGrant);result["secondSignature"]=secondSignature
    result["secondSignatureVerified"]=verify(secondSignature,"synthetic scoped payload")
    result["revoke"]=call(["op":"revoke-session","instance":newInstance,"capability":secondGrant["capability"] ?? "missing"])
    result["afterRevoke"]=session(newInstance,secondGrant)
    let lastGrant=open(newInstance,1);result["lastGrant"]=lastGrant
    let lastSignature=session(newInstance,lastGrant);result["lastSignature"]=lastSignature
    result["lastSignatureVerified"]=verify(lastSignature,"synthetic scoped payload")
    result["afterExhaustion"]=session(newInstance,lastGrant)
    return result
}
#endif

func runFixture() throws {
    #if CUSTODIAN
    // No command/config/environment override route exists in the custodian.
    guard CommandLine.arguments.count == 1 else { emit(["refused": "arguments"]); exit(64) }
    try validateCustodianInstallation()
    let c = try config(); let delegate = Custodian(c)
    let listener = NSXPCListener(machServiceName: c["machService"] as! String)
    listener.setConnectionCodeSigningRequirement(requirement(c["controllerID"] as! String, team: c["team"] as! String))
    let initialKey = keyPreflight(c) // Complete key initialization before opening admission.
    let backend=try RustCustody.initialize(c,newlyCreatedKey:initialKey["existing"] as? Bool==false)
    guard backend["ok"] as? Bool==true else {emit(["refused":"castellan-initialization","backend":backend]);exit(1)}
    listener.delegate = delegate; listener.resume()
    emit(["ready": true, "pid": getpid(), "ppid": getppid(), "key": initialKey,"castellan":backend])
    withExtendedLifetime((listener, delegate)) { RunLoop.current.run() }
    #elseif WORKER
    let delegate = Worker(); let listener = NSXPCListener.service()
    listener.delegate = delegate; listener.resume(); withExtendedLifetime(delegate) { RunLoop.current.run() }
    #else
    var c = try config(); let mode = CommandLine.arguments.dropFirst().first ?? "inspect"
    if CommandLine.arguments.count > 2, let pid = Int(CommandLine.arguments[2]) { c["custodianPID"] = pid }
    if CommandLine.arguments.count > 3, let port = Int(CommandLine.arguments[3]) { c["networkPort"] = port }
    switch mode {
    #if CONTROLLER
    case "castellan-e2e": emit(castellanE2E(c))
    #endif
    case "inspect": emit(workerProbe(c))
    case "key-preflight": emit(keyPreflight(c))
    case "key-cleanup": emit(["deleteStatus": SecItemDelete(keyQuery(c) as CFDictionary)])
    case "worker":
        let fd = open(c["vaultCanary"] as! String, O_RDONLY) // Intentionally NOT CLOEXEC: prove launchd does not inherit it.
        defer { if fd >= 0 { close(fd) } }
        let conn = NSXPCConnection(serviceName: c["workerID"] as! String)
        conn.setCodeSigningRequirement(requirement(c["workerID"] as! String, team: c["team"] as! String))
        conn.resume(); defer { conn.invalidate() }
        emit(rpc(conn, c))
    case "register":
        let service = SMAppService.agent(plistName: c["agentPlist"] as! String)
        if service.status != .enabled && service.status != .requiresApproval { try service.register() }
        emit(["registration": true, "status": service.status.rawValue])
    case "unregister":
        try SMAppService.agent(plistName: c["agentPlist"] as! String).unregister(); emit(["unregistered": true])
    case "client", "fake-client", "unsafe-fake-control", "cleanup-key":
        let unsafeControl = mode == "unsafe-fake-control"
        let conn: NSXPCConnection
        if unsafeControl {
            // Deliberately broken control: prove the fake can capture fixture data if peer pinning is removed.
            conn = NSXPCConnection(machServiceName: c["fakeService"] as! String); conn.resume()
        } else {
            conn = custodyConnection(c, service: mode == "fake-client" ? c["fakeService"] as? String : nil)
        }
        defer { conn.invalidate() }
        let lane = unsafeControl ? "unsafe-control" : "pinned"
        var hello = rpc(conn, ["op": "hello", "lane": lane])
        guard hello["ok"] as? Bool == true, hello["runID"] as? String == c["runID"] as? String,
              let instance = hello["instance"] as? String else {
            emit(["hello": hello, "secretTransmitted": false]); return
        }
        if mode == "cleanup-key" {
            emit(rpc(conn, ["op": "fixture-delete-key", "instance": instance])); return
        }
        if let state=hello["castellan"] as? [String:Any],state["locked"] as? Bool==true {
            let unlocked=rpc(conn,["op":"native-unlock","instance":instance],timeout:120)
            guard unlocked["ok"] as? Bool==true else {emit(["hello":hello,"nativeUnlock":unlocked,"signatureVerified":false]);return}
            hello=rpc(conn,["op":"hello"])
        }
        let challenge = "synthetic-only-" + UUID().uuidString
        let signed = rpc(conn, ["op": "sign", "namespace": "h1.root-sign.v1", "instance": instance,
                                "challenge": challenge, "lane": lane], timeout: 120)
        var verified = false
        if let pub = signed["publicKey"] as? String, let p = Data(base64Encoded: pub),
           let sig = signed["signature"] as? String, let data = Data(base64Encoded: sig),
           let publicKey = try? Curve25519.Signing.PublicKey(rawRepresentation: p) {
            verified = publicKey.isValidSignature(data, for: Data(challenge.utf8))
        }
        let denied = rpc(conn, ["op": "sign", "namespace": "other-profile", "instance": instance, "challenge": "synthetic-adjacent-scope"])
        emit(["hello": hello, "signatureVerified": verified, "publicKey": signed["publicKey"] ?? "",
              "sign": signed, "adjacentScope": denied, "secretTransmitted": true])
    #if OUTSIDER
    case "signer-oracle":
        // Fixed synthetic target only. No private key is exported or ACL changed.
        let signer = Process(); signer.executableURL = URL(fileURLWithPath: "/usr/bin/codesign")
        signer.arguments = ["--force", "--sign", c["signingIdentity"] as! String,
                            "--options", "runtime", "--timestamp=none", "--entitlements",
                            c["forgedEntitlements"] as! String, c["forgedApp"] as! String]
        let output = Pipe(); signer.standardOutput = output; signer.standardError = output
        let start = Date(); try signer.run()
        while signer.isRunning && Date().timeIntervalSince(start) < 8 { Thread.sleep(forTimeInterval: 0.02) }
        let timedOut = signer.isRunning
        if timedOut { kill(signer.processIdentifier, SIGKILL) }
        signer.waitUntilExit()
        let data = output.fileHandleForReading.readDataToEndOfFile()
        emit(["signerUnattended": !timedOut && signer.terminationStatus == 0,
              "timedOut": timedOut, "signerExit": signer.terminationStatus,
              "elapsed": Date().timeIntervalSince(start),
              "output": String(data: data, encoding: .utf8) ?? ""])
    case "fake-server":
        let delegate = FakeCustodian(); let listener = NSXPCListener(machServiceName: c["fakeService"] as! String)
        listener.delegate = delegate; listener.resume(); emit(["fakeReady": true])
        withExtendedLifetime((listener, delegate)) { RunLoop.current.run() }
    #endif
    default: emit(["refused": "unknown-mode"]); exit(64)
    }
    #endif
}
@main
struct FixtureEntry {
    static func main() {
        do { try runFixture() } catch { emit(["fatal": (error as NSError).code, "domain": (error as NSError).domain]); exit(1) }
    }
}
