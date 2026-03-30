import Cocoa
import UserNotifications

class AppDelegate: NSObject, NSApplicationDelegate, UNUserNotificationCenterDelegate {

    private var socketServer: SocketServer?
    private var statusItem: NSStatusItem!
    private var popover: NSPopover!
    private var popoverVC: PopoverViewController!

    func applicationDidFinishLaunching(_ notification: Notification) {
        ProcessInfo.processInfo.disableAutomaticTermination("Menu bar app")
        ProcessInfo.processInfo.disableSuddenTermination()

        // Initial data load
        BarkDataStore.shared.refresh()

        // Setup UI
        setupMenuBar()
        setupPopover()

        // Notifications
        let center = UNUserNotificationCenter.current()
        center.delegate = self
        registerCategories(center)
        center.requestAuthorization(options: [.alert, .sound, .badge]) { granted, error in
            if let error = error { NSLog("BarkNotifier: auth error: \(error)") }
            NSLog("BarkNotifier: notification permission \(granted ? "granted" : "denied")")
        }

        // Socket server
        let socketPath = Self.socketPath()
        socketServer = SocketServer(path: socketPath, delegate: self)
        socketServer?.start()

        // Listen for theme changes
        NotificationCenter.default.addObserver(self, selector: #selector(themeDidChange), name: .barkThemeChanged, object: nil)

        NSLog("BarkNotifier: started, socket at \(socketPath)")
    }

    func applicationWillTerminate(_ notification: Notification) {
        socketServer?.stop()
    }

    // MARK: - Menu Bar

    private func setupMenuBar() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        if let button = statusItem.button {
            button.image = makeStatusIcon()
            button.toolTip = "Bark Notifier"
            button.action = #selector(togglePopover(_:))
            button.target = self
        }
    }

    private func makeStatusIcon() -> NSImage {
        // Load the dog silhouette from Resources
        let bundle = Bundle.main
        if let path = bundle.path(forResource: "menubar", ofType: "png"),
           let img = NSImage(contentsOfFile: path) {
            img.size = NSSize(width: 20, height: 20)
            img.isTemplate = true
            return img
        }
        // Fallback: simple circle
        let size = NSSize(width: 18, height: 18)
        let img = NSImage(size: size)
        img.lockFocus()
        NSColor.black.setFill()
        NSBezierPath(ovalIn: NSRect(x: 3, y: 3, width: 12, height: 12)).fill()
        img.unlockFocus()
        img.isTemplate = true
        return img
    }

    // MARK: - Popover

    private func setupPopover() {
        popoverVC = PopoverViewController()

        popover = NSPopover()
        popover.contentViewController = popoverVC
        popover.behavior = .transient
        popover.contentSize = NSSize(width: BarkTheme.popoverWidth, height: BarkTheme.popoverHeight)
        applyThemeAppearance()
    }

    @objc private func togglePopover(_ sender: Any?) {
        if popover.isShown {
            popover.performClose(sender)
        } else {
            if let button = statusItem.button {
                // Refresh data before showing
                BarkDataStore.shared.isRunning = FileManager.default.fileExists(atPath: Self.socketPath())
                BarkDataStore.shared.refresh()
                popoverVC.refreshAll()
                popover.show(relativeTo: button.bounds, of: button, preferredEdge: .minY)
            }
        }
    }

    @objc private func themeDidChange() {
        applyThemeAppearance()
    }

    private func applyThemeAppearance() {
        popover.appearance = BarkTheme.mode.appearance
    }

    // MARK: - Stats (called by SocketServer)

    func recordAssessment(risk: String, description: String) {
        let store = BarkDataStore.shared
        store.sessionAssessments += 1
        if risk.uppercased() == "HIGH" { store.sessionHighRisk += 1 }

        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm:ss"
        let time = formatter.string(from: Date())

        DispatchQueue.main.async {
            self.popoverVC.updateStats(
                total: store.sessionAssessments, highRisk: store.sessionHighRisk,
                allowed: store.sessionAllowed, denied: store.sessionDenied
            )
            self.popoverVC.appendLog(time: time, risk: risk, description: description)
        }
    }

    func recordDecision(_ action: String) {
        let store = BarkDataStore.shared
        switch action {
        case "allow": store.sessionAllowed += 1
        case "deny":  store.sessionDenied += 1
        default: break
        }

        DispatchQueue.main.async {
            self.popoverVC.updateStats(
                total: store.sessionAssessments, highRisk: store.sessionHighRisk,
                allowed: store.sessionAllowed, denied: store.sessionDenied
            )
        }
    }

    // MARK: - Notification Categories

    private func registerCategories(_ center: UNUserNotificationCenter) {
        let allowAction = UNNotificationAction(identifier: "ALLOW", title: "Allow", options: [])
        let denyAction = UNNotificationAction(identifier: "DENY", title: "Deny", options: [.destructive])
        let skipAction = UNNotificationAction(identifier: "SKIP", title: "Skip to Terminal", options: [])

        let highRiskCategory = UNNotificationCategory(
            identifier: "HIGH_RISK",
            actions: [allowAction, denyAction, skipAction],
            intentIdentifiers: [],
            options: [.customDismissAction]
        )
        let infoCategory = UNNotificationCategory(identifier: "INFO", actions: [], intentIdentifiers: [], options: [])
        center.setNotificationCategories([highRiskCategory, infoCategory])
    }

    // MARK: - UNUserNotificationCenterDelegate

    func userNotificationCenter(_ center: UNUserNotificationCenter, didReceive response: UNNotificationResponse,
                                withCompletionHandler completionHandler: @escaping () -> Void) {
        let id = response.notification.request.identifier
        let action: String
        switch response.actionIdentifier {
        case "ALLOW": action = "allow"
        case "DENY":  action = "deny"
        case "SKIP":  action = "skip"
        case UNNotificationDismissActionIdentifier: action = "skip"
        default:
            // User tapped the notification body — skip + focus terminal
            action = "skip"
            activateTerminal()
        }
        NSLog("BarkNotifier: action=\(action) id=\(id)")
        recordDecision(action)
        socketServer?.sendDecision(id: id, action: action)
        completionHandler()
    }

    func userNotificationCenter(_ center: UNUserNotificationCenter, willPresent notification: UNNotification,
                                withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions) -> Void) {
        completionHandler([.banner, .sound])
    }

    // MARK: - Terminal Activation

    /// Activate the frontmost terminal app window.
    /// Tries common terminal apps in order of likelihood.
    private func activateTerminal() {
        let terminalBundleIDs = [
            "com.mitchellh.ghostty",           // Ghostty
            "com.googlecode.iterm2",           // iTerm2
            "dev.warp.Warp-Stable",            // Warp
            "com.apple.Terminal",              // Terminal.app
            "co.zeit.hyper",                   // Hyper
            "com.github.alacritty",            // Alacritty
            "net.kovidgoyal.kitty",            // Kitty
        ]

        let workspace = NSWorkspace.shared
        for bundleID in terminalBundleIDs {
            if let app = workspace.runningApplications.first(where: { $0.bundleIdentifier == bundleID }) {
                app.activate()
                NSLog("BarkNotifier: activated \(bundleID)")
                return
            }
        }
        NSLog("BarkNotifier: no known terminal app found to activate")
    }

    static func socketPath() -> String {
        let home = FileManager.default.homeDirectoryForCurrentUser.path
        return "\(home)/.claude/bark-notifier.sock"
    }
}
