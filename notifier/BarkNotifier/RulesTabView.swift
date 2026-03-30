import Cocoa

class RulesTabView: NSView {

    private var scrollView: NSScrollView!
    private var rulesStack: NSStackView!
    private var emptyContainer: NSView!
    private var titleLabel: NSTextField!
    private var editBtn: NSButton!
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

        // ── Header ──
        y -= 38
        titleLabel = makeLabel("Custom Rules", font: BarkTheme.headerTitle, color: BarkTheme.primaryText,
                               frame: NSRect(x: P, y: y + 8, width: 160, height: 20))
        addSubview(titleLabel)

        editBtn = NSButton(title: "Edit", target: self, action: #selector(editRules))
        editBtn.bezelStyle = .inline
        editBtn.font = BarkTheme.buttonFont
        editBtn.contentTintColor = BarkTheme.brand
        editBtn.frame = NSRect(x: W - P - 50, y: y + 6, width: 50, height: 24)
        addSubview(editBtn)

        // ── Separator ──
        y -= 4
        sep = makeSeparator(y: y, width: W)
        addSubview(sep)
        y -= 1

        // ── Rules Scroll ──
        rulesStack = NSStackView()
        rulesStack.orientation = .vertical
        rulesStack.alignment = .leading
        rulesStack.spacing = 6
        rulesStack.translatesAutoresizingMaskIntoConstraints = false

        let docView = FlippedView()
        docView.translatesAutoresizingMaskIntoConstraints = false
        docView.addSubview(rulesStack)

        scrollView = NSScrollView(frame: NSRect(x: 0, y: 0, width: W, height: y))
        scrollView.hasVerticalScroller = true
        scrollView.drawsBackground = false
        scrollView.documentView = docView
        scrollView.scrollerStyle = .overlay
        addSubview(scrollView)

        NSLayoutConstraint.activate([
            rulesStack.topAnchor.constraint(equalTo: docView.topAnchor, constant: 8),
            rulesStack.leadingAnchor.constraint(equalTo: docView.leadingAnchor, constant: P),
            rulesStack.trailingAnchor.constraint(equalTo: docView.trailingAnchor, constant: -P),
            rulesStack.bottomAnchor.constraint(equalTo: docView.bottomAnchor),
            rulesStack.widthAnchor.constraint(equalToConstant: W - P * 2),
        ])

        // Empty state
        emptyContainer = NSView(frame: NSRect(x: 0, y: y / 2 - 40, width: W, height: 80))

        let emptyText = makeLabel("No custom rules defined", font: BarkTheme.body, color: BarkTheme.dimText,
                                   frame: NSRect(x: 0, y: 40, width: W, height: 20))
        emptyText.alignment = .center
        emptyContainer.addSubview(emptyText)

        let createBtn = NSButton(title: "Create bark.toml", target: self, action: #selector(createRulesFile))
        createBtn.bezelStyle = .rounded
        createBtn.font = BarkTheme.buttonFont
        createBtn.frame = NSRect(x: (W - 130) / 2, y: 6, width: 130, height: 28)
        emptyContainer.addSubview(createBtn)

        emptyContainer.isHidden = true
        addSubview(emptyContainer)
    }

    func reload() {
        let rules = BarkDataStore.shared.rules

        rulesStack.arrangedSubviews.forEach {
            rulesStack.removeArrangedSubview($0)
            $0.removeFromSuperview()
        }

        emptyContainer.isHidden = !rules.isEmpty
        scrollView.isHidden = rules.isEmpty

        for rule in rules {
            let card = RuleCardView(frame: NSRect(x: 0, y: 0, width: frame.width - BarkTheme.padding * 2, height: 72), rule: rule)
            rulesStack.addArrangedSubview(card)
            card.widthAnchor.constraint(equalToConstant: frame.width - BarkTheme.padding * 2).isActive = true
            card.heightAnchor.constraint(equalToConstant: 72).isActive = true
        }
    }

    @objc private func themeChanged() {
        titleLabel.textColor = BarkTheme.primaryText
        sep.layer?.backgroundColor = BarkTheme.separator.cgColor
        reload()  // Rule cards rebuild with fresh colors
    }

    @objc private func editRules() {
        let path = BarkDataStore.shared.tomlPath
        if !FileManager.default.fileExists(atPath: path) {
            createRulesFile()
        }
        NSWorkspace.shared.open(URL(fileURLWithPath: path))
    }

    @objc private func createRulesFile() {
        let path = BarkDataStore.shared.tomlPath
        let template = """
        # Bark Custom Rules
        # Each rule: name, risk ("low"/"medium"/"high"), reason
        # [rules.match]: tool, command (glob), file_path (glob)

        # Example: Allow cargo commands
        # [[rules]]
        # name = "allow-cargo"
        # risk = "low"
        # reason = "Cargo commands are generally safe"
        # [rules.match]
        # tool = "Bash"
        # command = "cargo *"
        """
        try? template.write(toFile: path, atomically: true, encoding: .utf8)
        NSWorkspace.shared.open(URL(fileURLWithPath: path))
    }
}

// MARK: - Rule Card

class RuleCardView: NSView {

    init(frame: NSRect, rule: ParsedRule) {
        super.init(frame: frame)
        wantsLayer = true
        layer?.backgroundColor = BarkTheme.cardBackground.cgColor
        layer?.cornerRadius = 6
        layer?.borderWidth = 0.5
        layer?.borderColor = BarkTheme.cardBorder.cgColor

        let P: CGFloat = 10

        // Line 1: Risk badge + name
        let badge = RiskBadge(frame: NSRect(x: P, y: frame.height - 24, width: 30, height: 16), risk: rule.risk)
        addSubview(badge)

        let name = makeLabel(rule.name, font: BarkTheme.bodyBold, color: BarkTheme.primaryText,
                              frame: NSRect(x: P + 36, y: frame.height - 25, width: frame.width - P * 2 - 36, height: 18))
        addSubview(name)

        // Line 2: Reason
        let reason = makeLabel(rule.reason, font: BarkTheme.small, color: BarkTheme.secondaryText,
                                frame: NSRect(x: P, y: frame.height - 44, width: frame.width - P * 2, height: 16))
        reason.lineBreakMode = .byTruncatingTail
        addSubview(reason)

        // Line 3: Match pattern
        var matchParts: [String] = []
        if let tool = rule.tool { matchParts.append("tool: \(tool)") }
        if let cmd = rule.command { matchParts.append("cmd: \(cmd)") }
        if let fp = rule.filePath { matchParts.append("path: \(fp)") }
        let matchText = matchParts.isEmpty ? "no match criteria" : matchParts.joined(separator: "  ")

        let match = makeLabel(matchText, font: BarkTheme.logDescription, color: BarkTheme.dimText,
                               frame: NSRect(x: P, y: 4, width: frame.width - P * 2, height: 16))
        match.lineBreakMode = .byTruncatingTail
        addSubview(match)
    }

    required init?(coder: NSCoder) { fatalError() }
}
