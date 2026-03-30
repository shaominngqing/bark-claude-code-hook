import Cocoa

class ActivityTabView: NSView {

    private var filterControl: NSSegmentedControl!
    private var clearButton: NSButton!
    private var scrollView: NSScrollView!
    private var logStack: NSStackView!
    private var emptyLabel: NSTextField!
    private var currentFilter: Int? = nil  // nil = ALL

    private var sep: NSView!

    override init(frame: NSRect) {
        super.init(frame: frame)
        setup()
        NotificationCenter.default.addObserver(self, selector: #selector(themeChanged), name: .barkThemeChanged, object: nil)
    }

    required init?(coder: NSCoder) { fatalError() }

    private func setup() {
        let W = frame.width
        let P = BarkTheme.padding
        var y = frame.height

        // ── Filter Bar ──
        y -= 38
        filterControl = NSSegmentedControl(labels: ["All", "Low", "Med", "High"], trackingMode: .selectOne, target: self, action: #selector(filterChanged(_:)))
        filterControl.selectedSegment = 0
        filterControl.frame = NSRect(x: P, y: y + 4, width: 200, height: 24)
        filterControl.segmentStyle = .capsule
        filterControl.font = NSFont.systemFont(ofSize: 11, weight: .medium)
        addSubview(filterControl)

        clearButton = NSButton(title: "Clear", target: self, action: #selector(clearLog))
        clearButton.bezelStyle = .inline
        clearButton.font = BarkTheme.buttonFont
        clearButton.contentTintColor = BarkTheme.secondaryText
        clearButton.frame = NSRect(x: W - P - 50, y: y + 4, width: 50, height: 24)
        addSubview(clearButton)

        // ── Separator ──
        y -= 4
        sep = makeSeparator(y: y, width: W)
        addSubview(sep)
        y -= 1

        // ── Log Scroll ──
        logStack = NSStackView()
        logStack.orientation = .vertical
        logStack.alignment = .leading
        logStack.spacing = 1
        logStack.translatesAutoresizingMaskIntoConstraints = false

        let docView = FlippedView()
        docView.translatesAutoresizingMaskIntoConstraints = false
        docView.addSubview(logStack)

        scrollView = NSScrollView(frame: NSRect(x: 0, y: 0, width: W, height: y))
        scrollView.hasVerticalScroller = true
        scrollView.drawsBackground = false
        scrollView.documentView = docView
        scrollView.scrollerStyle = .overlay
        addSubview(scrollView)

        NSLayoutConstraint.activate([
            logStack.topAnchor.constraint(equalTo: docView.topAnchor),
            logStack.leadingAnchor.constraint(equalTo: docView.leadingAnchor),
            logStack.trailingAnchor.constraint(equalTo: docView.trailingAnchor),
            logStack.bottomAnchor.constraint(equalTo: docView.bottomAnchor),
            logStack.widthAnchor.constraint(equalToConstant: W),
        ])

        // Empty state
        emptyLabel = makeLabel("No activity yet", font: BarkTheme.body, color: BarkTheme.dimText,
                                frame: NSRect(x: 0, y: y / 2 - 10, width: W, height: 20))
        emptyLabel.alignment = .center
        addSubview(emptyLabel)
    }

    // MARK: - Data Loading

    func reload() {
        let entries = SQLiteReader.shared.getLog(count: 100, riskFilter: currentFilter)
        rebuildLog(entries)
    }

    private func rebuildLog(_ entries: [LogEntry]) {
        logStack.arrangedSubviews.forEach {
            logStack.removeArrangedSubview($0)
            $0.removeFromSuperview()
        }

        emptyLabel.isHidden = !entries.isEmpty

        for entry in entries {
            let row = ActivityLogRow(
                frame: NSRect(x: 0, y: 0, width: frame.width, height: BarkTheme.rowHeight),
                entry: entry
            )
            logStack.addArrangedSubview(row)
            row.widthAnchor.constraint(equalToConstant: frame.width).isActive = true
            row.heightAnchor.constraint(equalToConstant: BarkTheme.rowHeight).isActive = true
        }

    }

    // MARK: - Actions

    @objc private func filterChanged(_ sender: NSSegmentedControl) {
        switch sender.selectedSegment {
        case 0: currentFilter = nil
        case 1: currentFilter = 0   // LOW
        case 2: currentFilter = 1   // MEDIUM
        case 3: currentFilter = 2   // HIGH
        default: currentFilter = nil
        }
        reload()
    }

    @objc private func clearLog() {
        SQLiteReader.shared.clearLog()
        reload()
    }

    @objc private func themeChanged() {
        clearButton.contentTintColor = BarkTheme.secondaryText
        emptyLabel.textColor = BarkTheme.dimText
        sep.layer?.backgroundColor = BarkTheme.separator.cgColor
        reload()
    }
}

// MARK: - Activity Log Row

class ActivityLogRow: NSView {

    init(frame: NSRect, entry: LogEntry) {
        super.init(frame: frame)

        let P = BarkTheme.padding
        var x: CGFloat = P

        // Timestamp
        let ts = String(entry.timestamp.suffix(8))  // HH:mm:ss
        let timeLabel = NSTextField(labelWithString: ts)
        timeLabel.font = BarkTheme.logTimestamp
        timeLabel.textColor = BarkTheme.dimText
        timeLabel.frame = NSRect(x: x, y: 5, width: 58, height: 18)
        addSubview(timeLabel)
        x += 62

        // Risk badge
        let badge = RiskBadge(frame: NSRect(x: x, y: 5, width: 30, height: 18), risk: entry.riskString)
        addSubview(badge)
        x += 34

        // Source tag
        let srcLabel = NSTextField(labelWithString: entry.source)
        srcLabel.font = BarkTheme.logSource
        srcLabel.textColor = BarkTheme.dimText
        srcLabel.frame = NSRect(x: x, y: 6, width: 40, height: 16)
        addSubview(srcLabel)
        x += 44

        // Description
        let desc = entry.displayDescription
        let descLabel = NSTextField(labelWithString: desc)
        descLabel.font = BarkTheme.logDescription
        descLabel.textColor = BarkTheme.primaryText
        descLabel.lineBreakMode = .byTruncatingTail
        let descW = frame.width - x - 40 - P
        descLabel.frame = NSRect(x: x, y: 5, width: descW, height: 18)
        addSubview(descLabel)

        // Duration
        let durLabel = NSTextField(labelWithString: "\(entry.durationMs)ms")
        durLabel.font = BarkTheme.logTimestamp
        durLabel.textColor = BarkTheme.dimText
        durLabel.alignment = .right
        durLabel.frame = NSRect(x: frame.width - P - 38, y: 6, width: 38, height: 16)
        addSubview(durLabel)
    }

    required init?(coder: NSCoder) { fatalError() }
}
