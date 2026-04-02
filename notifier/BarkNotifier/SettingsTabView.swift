import Cocoa

class SettingsTabView: NSView {

    private var hookSwitch: NSSwitch!
    private var hookTitleLabel: NSTextField!
    private var hookLabel: NSTextField!
    private var themeTitleLabel: NSTextField!
    private var themeControl: NSSegmentedControl!
    private var timeoutTitleLabel: NSTextField!
    private var timeoutValueLabel: NSTextField!
    private var cacheTitleLabel: NSTextField!
    private var cacheLabel: NSTextField!
    private var clearCacheBtn: NSButton!
    private var versionTitleLabel: NSTextField!
    private var versionValueLabel: NSTextField!
    private var quitBtn: NSButton!
    private var separators: [NSView] = []

    override init(frame: NSRect) {
        super.init(frame: frame)
        setup()
        NotificationCenter.default.addObserver(self, selector: #selector(onThemeNotification), name: .barkThemeChanged, object: nil)
    }

    required init?(coder: NSCoder) { fatalError() }

    private func setup() {
        let W = frame.width
        let P = BarkTheme.padding
        var y = frame.height

        // ── Hook Toggle ──
        y -= 52
        hookTitleLabel = makeLabel("Bark Hook", font: BarkTheme.bodyBold, color: BarkTheme.primaryText,
                                    frame: NSRect(x: P, y: y + 14, width: 120, height: 18))
        addSubview(hookTitleLabel)

        hookLabel = makeLabel("Enabled", font: BarkTheme.small, color: BarkTheme.running,
                               frame: NSRect(x: P, y: y, width: 120, height: 16))
        addSubview(hookLabel)

        hookSwitch = NSSwitch()
        hookSwitch.frame = NSRect(x: W - P - 40, y: y + 8, width: 40, height: 24)
        hookSwitch.target = self
        hookSwitch.action = #selector(hookToggled(_:))
        addSubview(hookSwitch)

        y -= 8
        separators.append(addSep(y: y, width: W))
        y -= 1

        // ── Theme ──
        y -= 48
        themeTitleLabel = makeLabel("Appearance", font: BarkTheme.bodyBold, color: BarkTheme.primaryText,
                                     frame: NSRect(x: P, y: y + 14, width: 120, height: 18))
        addSubview(themeTitleLabel)

        themeControl = NSSegmentedControl(labels: ["Light", "Dark", "System"], trackingMode: .selectOne, target: self, action: #selector(themeChanged(_:)))
        themeControl.segmentStyle = .capsule
        themeControl.font = NSFont.systemFont(ofSize: 11, weight: .medium)
        themeControl.frame = NSRect(x: W - P - 180, y: y + 10, width: 180, height: 24)
        switch BarkTheme.mode {
        case .light:  themeControl.selectedSegment = 0
        case .dark:   themeControl.selectedSegment = 1
        case .system: themeControl.selectedSegment = 2
        }
        addSubview(themeControl)

        y -= 8
        separators.append(addSep(y: y, width: W))
        y -= 1

        // ── Auto-skip timeout ──
        y -= 40
        timeoutTitleLabel = makeLabel("Auto-skip Timeout", font: BarkTheme.body, color: BarkTheme.primaryText,
                                       frame: NSRect(x: P, y: y + 10, width: 160, height: 18))
        addSubview(timeoutTitleLabel)

        timeoutValueLabel = makeLabel("10 seconds", font: BarkTheme.body, color: BarkTheme.secondaryText,
                                       frame: NSRect(x: W - P - 100, y: y + 10, width: 100, height: 18))
        timeoutValueLabel.alignment = .right
        addSubview(timeoutValueLabel)

        y -= 8
        separators.append(addSep(y: y, width: W))
        y -= 1

        // ── Cache ──
        y -= 48
        cacheTitleLabel = makeLabel("Cache", font: BarkTheme.bodyBold, color: BarkTheme.primaryText,
                                     frame: NSRect(x: P, y: y + 14, width: 80, height: 18))
        addSubview(cacheTitleLabel)

        cacheLabel = makeLabel("0 entries", font: BarkTheme.small, color: BarkTheme.secondaryText,
                                frame: NSRect(x: P, y: y, width: 160, height: 16))
        addSubview(cacheLabel)

        clearCacheBtn = NSButton(title: "Clear", target: self, action: #selector(clearCache))
        clearCacheBtn.bezelStyle = .inline
        clearCacheBtn.font = BarkTheme.buttonFont
        clearCacheBtn.contentTintColor = BarkTheme.secondaryText
        clearCacheBtn.frame = NSRect(x: W - P - 50, y: y + 6, width: 50, height: 24)
        addSubview(clearCacheBtn)

        y -= 8
        separators.append(addSep(y: y, width: W))
        y -= 1

        // ── Version ──
        y -= 36
        versionTitleLabel = makeLabel("Version", font: BarkTheme.body, color: BarkTheme.primaryText,
                                       frame: NSRect(x: P, y: y + 8, width: 80, height: 18))
        addSubview(versionTitleLabel)

        let appVersion = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "unknown"
        versionValueLabel = makeLabel(appVersion, font: BarkTheme.body, color: BarkTheme.secondaryText,
                                       frame: NSRect(x: W - P - 100, y: y + 8, width: 100, height: 18))
        versionValueLabel.alignment = .right
        addSubview(versionValueLabel)

        y -= 16
        separators.append(addSep(y: y, width: W))
        y -= 1

        // ── Quit ──
        quitBtn = NSButton(title: "Quit Bark Notifier", target: self, action: #selector(quitApp))
        quitBtn.bezelStyle = .inline
        quitBtn.isBordered = false
        quitBtn.font = BarkTheme.buttonFont
        quitBtn.contentTintColor = BarkTheme.secondaryText
        quitBtn.frame = NSRect(x: 0, y: 0, width: W, height: 40)
        addSubview(quitBtn)
    }

    private func addSep(y: CGFloat, width: CGFloat) -> NSView {
        let sep = makeSeparator(y: y, width: width)
        addSubview(sep)
        return sep
    }

    func reload() {
        let store = BarkDataStore.shared

        if !store.barkInstalled {
            hookSwitch.state = .off
            hookSwitch.isEnabled = false
            hookLabel.stringValue = "Bark not installed"
            hookLabel.textColor = BarkTheme.dimText
        } else {
            hookSwitch.isEnabled = true
            hookSwitch.state = store.hookEnabled ? .on : .off
            hookLabel.stringValue = store.hookEnabled ? "Enabled" : "Disabled"
            hookLabel.textColor = store.hookEnabled ? BarkTheme.running : BarkTheme.stopped
        }

        let cs = store.cacheStats
        let sizeKB = cs.sizeBytes / 1024
        cacheLabel.stringValue = "\(cs.count) entries, \(sizeKB) KB"
    }

    // MARK: - Theme Refresh

    @objc private func onThemeNotification() {
        hookTitleLabel.textColor = BarkTheme.primaryText
        themeTitleLabel.textColor = BarkTheme.primaryText
        timeoutTitleLabel.textColor = BarkTheme.primaryText
        timeoutValueLabel.textColor = BarkTheme.secondaryText
        cacheTitleLabel.textColor = BarkTheme.primaryText
        cacheLabel.textColor = BarkTheme.secondaryText
        versionTitleLabel.textColor = BarkTheme.primaryText
        versionValueLabel.textColor = BarkTheme.secondaryText
        clearCacheBtn.contentTintColor = BarkTheme.secondaryText
        quitBtn.contentTintColor = BarkTheme.secondaryText

        for sep in separators {
            sep.layer?.backgroundColor = BarkTheme.separator.cgColor
        }

        // Refresh hook label color
        let store = BarkDataStore.shared
        hookLabel.textColor = store.hookEnabled ? BarkTheme.running : BarkTheme.stopped
    }

    // MARK: - Actions

    @objc private func hookToggled(_ sender: NSSwitch) {
        let enable = sender.state == .on
        let cmd = enable ? "on" : "off"

        let barkPath = findBarkBinary()
        let task = Process()
        task.executableURL = URL(fileURLWithPath: barkPath)
        task.arguments = [cmd]
        task.standardOutput = FileHandle.nullDevice
        task.standardError = FileHandle.nullDevice
        try? task.run()
        task.waitUntilExit()

        BarkDataStore.shared.refresh()
        reload()
        NotificationCenter.default.post(name: .barkHookStateChanged, object: nil)
    }

    @objc private func themeChanged(_ sender: NSSegmentedControl) {
        switch sender.selectedSegment {
        case 0: BarkTheme.mode = .light
        case 1: BarkTheme.mode = .dark
        case 2: BarkTheme.mode = .system
        default: break
        }
    }

    @objc private func clearCache() {
        SQLiteReader.shared.clearCache()
        BarkDataStore.shared.refresh()
        reload()
    }

    @objc private func quitApp() {
        NSApplication.shared.terminate(nil)
    }

    private func findBarkBinary() -> String {
        let candidates = [
            "/opt/homebrew/bin/bark",
            "/usr/local/bin/bark",
            "\(FileManager.default.homeDirectoryForCurrentUser.path)/.local/bin/bark",
            "\(FileManager.default.homeDirectoryForCurrentUser.path)/.cargo/bin/bark",
        ]
        for path in candidates {
            if FileManager.default.fileExists(atPath: path) { return path }
        }
        let task = Process()
        task.executableURL = URL(fileURLWithPath: "/usr/bin/which")
        task.arguments = ["bark"]
        let pipe = Pipe()
        task.standardOutput = pipe
        try? task.run()
        task.waitUntilExit()
        let output = String(data: pipe.fileHandleForReading.readDataToEndOfFile(), encoding: .utf8)?
            .trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        return output.isEmpty ? "/usr/local/bin/bark" : output
    }
}
