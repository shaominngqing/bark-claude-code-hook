import Cocoa

class DashboardTabView: NSView {

    private var statusDot: NSView!
    private var statusLabel: NSTextField!
    private var versionLabel: NSTextField!
    private var assessmentsCard: StatCardView!
    private var highRiskCard: StatCardView!
    private var allowedCard: StatCardView!
    private var deniedCard: StatCardView!
    private var riskBar: RiskDistributionBar!
    private var separators: [NSView] = []
    private var sectionLabels: [NSTextField] = []

    override init(frame: NSRect) {
        super.init(frame: frame)
        setup()
        NotificationCenter.default.addObserver(self, selector: #selector(themeChanged), name: .barkThemeChanged, object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(hookStateChanged), name: .barkHookStateChanged, object: nil)
    }

    required init?(coder: NSCoder) { fatalError() }

    private func setup() {
        let W = frame.width
        let P = BarkTheme.padding
        var y = frame.height

        // ── Status Row ──
        y -= 36
        statusDot = NSView(frame: NSRect(x: P, y: y + 10, width: 8, height: 8))
        statusDot.wantsLayer = true
        statusDot.layer?.cornerRadius = 4
        statusDot.layer?.backgroundColor = BarkTheme.running.cgColor
        addSubview(statusDot)

        statusLabel = makeLabel("Running", font: BarkTheme.statusLabel, color: BarkTheme.running,
                                frame: NSRect(x: P + 14, y: y + 6, width: 80, height: 16))
        addSubview(statusLabel)

        versionLabel = makeLabel("v2.0.2", font: BarkTheme.small, color: BarkTheme.dimText,
                                  frame: NSRect(x: W - P - 50, y: y + 6, width: 50, height: 16))
        versionLabel.alignment = .right
        addSubview(versionLabel)

        // ── Stat Cards 2×2 ──
        let cardW = (W - P * 2 - BarkTheme.cardSpacing) / 2
        y -= BarkTheme.cardHeight + 4

        assessmentsCard = StatCardView(
            frame: NSRect(x: P, y: y, width: cardW, height: BarkTheme.cardHeight),
            label: "ASSESSMENTS", accent: BarkTheme.accentAssessments
        )
        highRiskCard = StatCardView(
            frame: NSRect(x: P + cardW + BarkTheme.cardSpacing, y: y, width: cardW, height: BarkTheme.cardHeight),
            label: "HIGH RISK", accent: BarkTheme.accentHighRisk
        )
        addSubview(assessmentsCard)
        addSubview(highRiskCard)

        y -= BarkTheme.cardSpacing + BarkTheme.cardHeight
        allowedCard = StatCardView(
            frame: NSRect(x: P, y: y, width: cardW, height: BarkTheme.cardHeight),
            label: "ALLOWED", accent: BarkTheme.accentAllowed
        )
        deniedCard = StatCardView(
            frame: NSRect(x: P + cardW + BarkTheme.cardSpacing, y: y, width: cardW, height: BarkTheme.cardHeight),
            label: "DENIED", accent: BarkTheme.accentDenied
        )
        addSubview(allowedCard)
        addSubview(deniedCard)

        // ── Risk Distribution ──
        y -= 10
        let sep1 = makeSeparator(y: y, width: W)
        separators.append(sep1)
        addSubview(sep1)
        y -= 1

        y -= 24
        let sectionLabel = makeLabel("RISK DISTRIBUTION", font: BarkTheme.sectionTitle, color: BarkTheme.sectionHeader,
                                      frame: NSRect(x: P, y: y, width: W - P * 2, height: 16))
        sectionLabels.append(sectionLabel)
        addSubview(sectionLabel)

        y -= 28
        riskBar = RiskDistributionBar(frame: NSRect(x: P, y: y, width: W - P * 2, height: 20))
        addSubview(riskBar)

        refreshFromStore()
    }

    // MARK: - Public

    func updateSessionStats(total: Int, highRisk: Int, allowed: Int, denied: Int) {
        assessmentsCard.setValue(total)
        highRiskCard.setValue(highRisk)
        allowedCard.setValue(allowed)
        deniedCard.setValue(denied)
    }

    func updateRunning(_ running: Bool) {
        let store = BarkDataStore.shared
        let color: NSColor
        let text: String

        if !store.barkInstalled {
            color = BarkTheme.dimText
            text = "Not Installed"
        } else if running {
            color = BarkTheme.running
            text = "Running"
        } else {
            color = BarkTheme.stopped
            text = "Stopped"
        }

        statusDot.layer?.backgroundColor = color.cgColor
        statusLabel.textColor = color
        statusLabel.stringValue = text
    }

    func refreshFromStore() {
        let store = BarkDataStore.shared
        let stats = store.aggregateStats

        // Use historical totals from SQLite
        let total = stats.total
        let high = stats.byLevel["HIGH"] ?? 0
        updateSessionStats(total: total, highRisk: high,
                           allowed: store.sessionAllowed, denied: store.sessionDenied)
        updateRunning(store.hookEnabled)

        riskBar.update(low: stats.byLevel["LOW"] ?? 0,
                       med: stats.byLevel["MEDIUM"] ?? 0,
                       high: high)
    }

    @objc private func hookStateChanged() {
        BarkDataStore.shared.refresh()
        updateRunning(BarkDataStore.shared.hookEnabled)
    }

    @objc private func themeChanged() {
        versionLabel.textColor = BarkTheme.dimText
        for sep in separators {
            sep.layer?.backgroundColor = BarkTheme.separator.cgColor
        }
        for label in sectionLabels {
            label.textColor = BarkTheme.sectionHeader
        }
        assessmentsCard.refreshTheme()
        highRiskCard.refreshTheme()
        allowedCard.refreshTheme()
        deniedCard.refreshTheme()
        riskBar.refreshTheme()
    }
}

// MARK: - Risk Distribution Bar

class RiskDistributionBar: NSView {

    private var lowBar: NSView!
    private var medBar: NSView!
    private var highBar: NSView!

    override init(frame: NSRect) {
        super.init(frame: frame)
        setup()
    }

    required init?(coder: NSCoder) { fatalError() }

    private func setup() {
        wantsLayer = true
        layer?.backgroundColor = BarkTheme.cardBackground.cgColor
        layer?.cornerRadius = 4

        lowBar = NSView()
        lowBar.wantsLayer = true
        lowBar.layer?.backgroundColor = NSColor.systemGreen.cgColor
        addSubview(lowBar)

        medBar = NSView()
        medBar.wantsLayer = true
        medBar.layer?.backgroundColor = NSColor.systemYellow.cgColor
        addSubview(medBar)

        highBar = NSView()
        highBar.wantsLayer = true
        highBar.layer?.backgroundColor = NSColor.systemRed.cgColor
        addSubview(highBar)

        update(low: 0, med: 0, high: 0)
    }

    func update(low: Int, med: Int, high: Int) {
        let total = max(low + med + high, 1)
        let w = frame.width
        let h: CGFloat = 8
        let y = (frame.height - h) / 2

        let lowW = w * CGFloat(low) / CGFloat(total)
        let medW = w * CGFloat(med) / CGFloat(total)
        let highW = w * CGFloat(high) / CGFloat(total)

        lowBar.frame = NSRect(x: 0, y: y, width: lowW, height: h)
        medBar.frame = NSRect(x: lowW, y: y, width: medW, height: h)
        highBar.frame = NSRect(x: lowW + medW, y: y, width: highW, height: h)

        lowBar.layer?.maskedCorners = [.layerMinXMinYCorner, .layerMinXMaxYCorner]
        lowBar.layer?.cornerRadius = 4
        highBar.layer?.maskedCorners = [.layerMaxXMinYCorner, .layerMaxXMaxYCorner]
        highBar.layer?.cornerRadius = 4
    }

    func refreshTheme() {
        layer?.backgroundColor = BarkTheme.cardBackground.cgColor
    }
}
