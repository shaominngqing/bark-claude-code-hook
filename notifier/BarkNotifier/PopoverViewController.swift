import Cocoa

// MARK: - PopoverViewController (Tab Container)

class PopoverViewController: NSViewController, TabBarDelegate {

    private var tabBar: TabBarView!
    private var contentArea: NSView!
    private var tabViews: [NSView] = []
    private var currentTabIndex: Int = 0

    // Tab view controllers
    var dashboardTab: DashboardTabView!
    var activityTab: ActivityTabView!
    var rulesTab: RulesTabView!
    var settingsTab: SettingsTabView!

    override func loadView() {
        let W = BarkTheme.popoverWidth
        let H = BarkTheme.popoverHeight

        let root = NSView(frame: NSRect(x: 0, y: 0, width: W, height: H))
        root.wantsLayer = true
        root.layer?.backgroundColor = BarkTheme.panelBackground.cgColor

        // Tab bar at top
        let tabH = BarkTheme.tabBarHeight
        tabBar = TabBarView(frame: NSRect(x: 0, y: H - tabH, width: W, height: tabH))
        tabBar.delegate = self
        root.addSubview(tabBar)

        // Separator
        let tabSep = makeSeparator(y: H - tabH - 1, width: W)
        tabSep.identifier = NSUserInterfaceItemIdentifier("tabSep")
        root.addSubview(tabSep)

        // Content area
        let contentH = H - tabH - 1
        contentArea = NSView(frame: NSRect(x: 0, y: 0, width: W, height: contentH))
        contentArea.wantsLayer = true
        root.addSubview(contentArea)

        // Create tab views
        let tabFrame = NSRect(x: 0, y: 0, width: W, height: contentH)
        dashboardTab = DashboardTabView(frame: tabFrame)
        activityTab = ActivityTabView(frame: tabFrame)
        rulesTab = RulesTabView(frame: tabFrame)
        settingsTab = SettingsTabView(frame: tabFrame)

        tabViews = [dashboardTab, activityTab, rulesTab, settingsTab]

        // Show first tab
        showTab(0)

        // Listen for theme changes
        NotificationCenter.default.addObserver(self, selector: #selector(themeChanged), name: .barkThemeChanged, object: nil)

        self.view = root
    }

    // MARK: - Tab Switching

    func tabBar(_ tabBar: TabBarView, didSelectTab index: Int) {
        showTab(index)
    }

    private func showTab(_ index: Int) {
        guard index >= 0 && index < tabViews.count else { return }

        // Remove current
        contentArea.subviews.forEach { $0.removeFromSuperview() }

        // Add new
        contentArea.addSubview(tabViews[index])
        currentTabIndex = index

        // Refresh data for the shown tab
        switch index {
        case 0: dashboardTab.refreshFromStore()
        case 1: activityTab.reload()
        case 2: rulesTab.reload()
        case 3: settingsTab.reload()
        default: break
        }
    }

    // MARK: - Public Methods (called by AppDelegate)

    func updateStats(total: Int, highRisk: Int, allowed: Int, denied: Int) {
        dashboardTab?.updateSessionStats(total: total, highRisk: highRisk, allowed: allowed, denied: denied)
    }

    func appendLog(time: String, risk: String, description: String) {
        // Refresh activity tab if visible
        if currentTabIndex == 1 {
            activityTab?.reload()
        }
    }

    func updateRunningStatus(_ running: Bool) {
        BarkDataStore.shared.isRunning = running
        dashboardTab?.updateRunning(running)
    }

    func refreshAll() {
        BarkDataStore.shared.refresh()
        dashboardTab?.refreshFromStore()
        if currentTabIndex == 1 { activityTab?.reload() }
        if currentTabIndex == 2 { rulesTab?.reload() }
        if currentTabIndex == 3 { settingsTab?.reload() }
    }

    @objc private func themeChanged() {
        view.layer?.backgroundColor = BarkTheme.panelBackground.cgColor
        tabBar.refreshColors()
        // Update separator
        if let sep = view.subviews.first(where: { $0.identifier?.rawValue == "tabSep" }) {
            sep.layer?.backgroundColor = BarkTheme.separator.cgColor
        }
        showTab(currentTabIndex)
    }
}

// MARK: - Shared Components

class StatCardView: NSView {

    private var valueLabel: NSTextField!
    private var nameLabel: NSTextField!
    private var accentLine: NSView!
    let accentColor: NSColor

    init(frame: NSRect, label: String, accent: NSColor) {
        self.accentColor = accent
        super.init(frame: frame)
        setup(label: label)
    }

    required init?(coder: NSCoder) { fatalError() }

    private func setup(label: String) {
        wantsLayer = true
        layer?.backgroundColor = BarkTheme.cardBackground.cgColor
        layer?.cornerRadius = BarkTheme.cornerRadius
        layer?.borderWidth = 0.5
        layer?.borderColor = BarkTheme.cardBorder.cgColor

        accentLine = NSView(frame: NSRect(x: 0, y: frame.height - 3, width: frame.width, height: 3))
        accentLine.wantsLayer = true
        accentLine.layer?.backgroundColor = accentColor.cgColor
        accentLine.layer?.maskedCorners = [.layerMinXMaxYCorner, .layerMaxXMaxYCorner]
        accentLine.layer?.cornerRadius = BarkTheme.cornerRadius
        addSubview(accentLine)

        valueLabel = NSTextField(labelWithString: "0")
        valueLabel.font = BarkTheme.statValue
        valueLabel.textColor = accentColor
        valueLabel.alignment = .center
        valueLabel.frame = NSRect(x: 0, y: 18, width: frame.width, height: 34)
        addSubview(valueLabel)

        nameLabel = NSTextField(labelWithString: label)
        nameLabel.font = BarkTheme.statLabel
        nameLabel.textColor = BarkTheme.secondaryText
        nameLabel.alignment = .center
        nameLabel.frame = NSRect(x: 0, y: 6, width: frame.width, height: 14)
        addSubview(nameLabel)
    }

    func setValue(_ n: Int) {
        valueLabel.stringValue = "\(n)"
    }

    func refreshTheme() {
        layer?.backgroundColor = BarkTheme.cardBackground.cgColor
        layer?.borderColor = BarkTheme.cardBorder.cgColor
        nameLabel.textColor = BarkTheme.secondaryText
    }
}

class RiskBadge: NSView {

    init(frame: NSRect, risk: String) {
        super.init(frame: frame)
        wantsLayer = true
        let color = BarkTheme.riskColor(risk)
        layer?.backgroundColor = color.withAlphaComponent(0.2).cgColor
        layer?.cornerRadius = 4

        let abbr: String
        switch risk.uppercased() {
        case "HIGH", "HIG", "HI": abbr = "HIG"
        case "MEDIUM", "MED":     abbr = "MED"
        default:                  abbr = "LOW"
        }

        let label = NSTextField(labelWithString: abbr)
        label.font = BarkTheme.logRiskBadge
        label.textColor = color
        label.alignment = .center
        label.frame = NSRect(x: 0, y: 1, width: frame.width, height: frame.height - 2)
        addSubview(label)
    }

    required init?(coder: NSCoder) { fatalError() }
}
