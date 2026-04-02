import Foundation
import UserNotifications

/// Listens on a Unix domain socket for notification requests from bark daemon.
///
/// Protocol: one-line JSON per request, one-line JSON per response, terminated by \n.
/// For "confirm" requests, holds the connection open until the user interacts
/// with the notification (or 5s auto-skip fires).
class SocketServer: NSObject {

    private let path: String
    private weak var delegate: AppDelegate?
    private var listenFd: Int32 = -1
    private var isRunning = false

    /// Connection handlers run on a concurrent queue
    private let workerQueue = DispatchQueue(
        label: "com.bark.notifier.workers",
        qos: .userInitiated,
        attributes: .concurrent
    )

    /// Pending confirm requests: id -> connection file descriptor
    private var pendingConfirms: [String: Int32] = [:]
    private let lock = NSLock()

    /// Auto-skip timers: id -> work item
    private var autoSkipTimers: [String: DispatchWorkItem] = [:]

    init(path: String, delegate: AppDelegate) {
        self.path = path
        self.delegate = delegate
        super.init()
    }

    func start() {
        // Set up the listening socket on the calling thread
        unlink(path)

        listenFd = Darwin.socket(AF_UNIX, SOCK_STREAM, 0)
        guard listenFd >= 0 else {
            NSLog("BarkNotifier: socket() failed: \(String(cString: strerror(errno)))")
            return
        }

        var addr = sockaddr_un()
        addr.sun_family = sa_family_t(AF_UNIX)
        withUnsafeMutablePointer(to: &addr.sun_path) { sunPathPtr in
            path.withCString { cstr in
                let len = min(Int(strlen(cstr)), 103)
                memcpy(sunPathPtr, cstr, len)
            }
        }

        let addrLen = socklen_t(MemoryLayout<sockaddr_un>.size)
        let bindResult = withUnsafePointer(to: &addr) { ptr in
            ptr.withMemoryRebound(to: sockaddr.self, capacity: 1) { sockPtr in
                Darwin.bind(listenFd, sockPtr, addrLen)
            }
        }

        guard bindResult == 0 else {
            NSLog("BarkNotifier: bind failed: \(String(cString: strerror(errno)))")
            Darwin.close(listenFd)
            return
        }

        guard Darwin.listen(listenFd, 5) == 0 else {
            NSLog("BarkNotifier: listen failed: \(String(cString: strerror(errno)))")
            Darwin.close(listenFd)
            return
        }

        isRunning = true

        // Run the blocking accept loop on a background thread
        let thread = Thread(target: self, selector: #selector(acceptLoop), object: nil)
        thread.qualityOfService = .userInitiated
        thread.name = "com.bark.notifier.accept"
        thread.start()
    }

    func stop() {
        isRunning = false
        unlink(path)
        if listenFd >= 0 {
            Darwin.close(listenFd)
            listenFd = -1
        }
        lock.lock()
        for (_, timer) in autoSkipTimers { timer.cancel() }
        autoSkipTimers.removeAll()
        for (_, fd) in pendingConfirms { Darwin.close(fd) }
        pendingConfirms.removeAll()
        lock.unlock()
    }

    /// Send a decision back to bark daemon for a pending confirm request.
    func sendDecision(id: String, action: String) {
        lock.lock()
        autoSkipTimers[id]?.cancel()
        autoSkipTimers.removeValue(forKey: id)
        guard let fd = pendingConfirms.removeValue(forKey: id) else {
            lock.unlock()
            return
        }
        lock.unlock()

        let response: [String: Any] = ["type": "decision", "id": id, "action": action]
        sendJSON(fd: fd, response)
        Darwin.close(fd)
    }

    // MARK: - Accept Loop

    @objc private func acceptLoop() {
        while isRunning {
            let clientFd = Darwin.accept(listenFd, nil, nil)
            guard clientFd >= 0 else {
                if isRunning {
                    NSLog("BarkNotifier: accept error: \(String(cString: strerror(errno)))")
                }
                continue
            }
            workerQueue.async { [weak self] in
                self?.handleConnection(clientFd)
            }
        }
    }

    // MARK: - Connection Handler

    private func handleConnection(_ fd: Int32) {
        guard let line = readLine(fd: fd) else {
            Darwin.close(fd)
            return
        }

        guard let data = line.data(using: .utf8),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let msgType = json["type"] as? String else {
            sendJSON(fd: fd, ["type": "error", "message": "invalid request"])
            Darwin.close(fd)
            return
        }

        switch msgType {
        case "ping":
            sendJSON(fd: fd, ["type": "pong"])
            Darwin.close(fd)

        case "info":
            let title = json["title"] as? String ?? "Bark"
            let body = json["body"] as? String ?? ""
            showInfoNotification(title: title, body: body)
            delegate?.recordAssessment(risk: "MEDIUM", description: body)
            sendJSON(fd: fd, ["type": "ack"])
            Darwin.close(fd)

        case "confirm":
            guard let id = json["id"] as? String else {
                sendJSON(fd: fd, ["type": "error", "message": "missing id"])
                Darwin.close(fd)
                return
            }
            let title = json["title"] as? String ?? "Bark"
            let body = json["body"] as? String ?? ""
            let reason = json["reason"] as? String ?? ""

            lock.lock()
            pendingConfirms[id] = fd
            lock.unlock()

            showConfirmNotification(id: id, title: title, body: body, reason: reason)
            delegate?.recordAssessment(risk: "HIGH", description: body)

            // Auto-skip after 5 seconds if user doesn't interact
            let autoSkip = DispatchWorkItem { [weak self] in
                self?.sendDecision(id: id, action: "skip")
            }
            lock.lock()
            autoSkipTimers[id] = autoSkip
            lock.unlock()
            DispatchQueue.main.asyncAfter(deadline: .now() + 3.0, execute: autoSkip)

        default:
            sendJSON(fd: fd, ["type": "error", "message": "unknown type: \(msgType)"])
            Darwin.close(fd)
        }
    }

    // MARK: - Notifications

    private func showInfoNotification(title: String, body: String) {
        let content = UNMutableNotificationContent()
        content.title = title
        content.body = body
        content.categoryIdentifier = "INFO"

        let req = UNNotificationRequest(identifier: UUID().uuidString, content: content, trigger: nil)
        UNUserNotificationCenter.current().add(req) { error in
            if let error = error {
                NSLog("BarkNotifier: info notification error: \(error)")
            }
        }
    }

    private func showConfirmNotification(id: String, title: String, body: String, reason: String) {
        let content = UNMutableNotificationContent()
        content.title = title
        content.subtitle = reason
        content.body = body
        content.sound = UNNotificationSound(named: UNNotificationSoundName("Funk"))
        content.categoryIdentifier = "HIGH_RISK"

        let req = UNNotificationRequest(identifier: id, content: content, trigger: nil)
        UNUserNotificationCenter.current().add(req) { error in
            if let error = error {
                NSLog("BarkNotifier: confirm notification error: \(error)")
            }
        }
    }

    // MARK: - Socket I/O

    private func readLine(fd: Int32) -> String? {
        var buf = [UInt8](repeating: 0, count: 4096)
        var data = Data()
        while true {
            let n = Darwin.read(fd, &buf, buf.count)
            if n <= 0 { break }
            data.append(contentsOf: buf[0..<n])
            if data.contains(0x0A) { break }
        }
        guard !data.isEmpty else { return nil }
        return String(data: data, encoding: .utf8)?.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private func sendJSON(fd: Int32, _ dict: [String: Any]) {
        guard let data = try? JSONSerialization.data(withJSONObject: dict),
              var str = String(data: data, encoding: .utf8) else { return }
        str += "\n"
        let bytes = Array(str.utf8)
        bytes.withUnsafeBufferPointer { buf in
            _ = Darwin.write(fd, buf.baseAddress!, buf.count)
        }
    }
}
