import Cocoa

protocol TabBarDelegate: AnyObject {
    func tabBar(_ tabBar: TabBarView, didSelectTab index: Int)
}

class TabBarView: NSView {

    weak var delegate: TabBarDelegate?
    private var buttons: [NSButton] = []
    private var indicator: NSView!
    private(set) var selectedIndex: Int = 0

    private let icons = [
        "chart.bar.fill",           // Dashboard
        "list.bullet.rectangle",    // Activity
        "text.badge.checkmark",     // Rules
        "gearshape.fill"            // Settings
    ]

    override init(frame: NSRect) {
        super.init(frame: frame)
        setup()
    }

    required init?(coder: NSCoder) { fatalError() }

    private func setup() {
        wantsLayer = true

        let count = icons.count
        let btnWidth = frame.width / CGFloat(count)

        for (i, iconName) in icons.enumerated() {
            let btn = NSButton(frame: NSRect(x: CGFloat(i) * btnWidth, y: 0, width: btnWidth, height: frame.height))
            btn.bezelStyle = .inline
            btn.isBordered = false
            btn.tag = i
            btn.target = self
            btn.action = #selector(tabTapped(_:))

            let config = NSImage.SymbolConfiguration(pointSize: 16, weight: .medium)
            if let img = NSImage(systemSymbolName: iconName, accessibilityDescription: nil)?
                .withSymbolConfiguration(config) {
                btn.image = img
                btn.imagePosition = .imageOnly
            }

            btn.contentTintColor = i == 0 ? BarkTheme.brand : BarkTheme.secondaryText
            buttons.append(btn)
            addSubview(btn)
        }

        // Selection indicator pill
        indicator = NSView(frame: NSRect(x: (btnWidth - 24) / 2, y: 2, width: 24, height: 3))
        indicator.wantsLayer = true
        indicator.layer?.backgroundColor = BarkTheme.brand.cgColor
        indicator.layer?.cornerRadius = 1.5
        addSubview(indicator)
    }

    func selectTab(_ index: Int) {
        guard index >= 0 && index < buttons.count else { return }
        selectedIndex = index

        let btnWidth = frame.width / CGFloat(buttons.count)
        for (i, btn) in buttons.enumerated() {
            btn.contentTintColor = i == index ? BarkTheme.brand : BarkTheme.secondaryText
        }

        NSAnimationContext.runAnimationGroup { ctx in
            ctx.duration = 0.15
            indicator.animator().frame.origin.x = CGFloat(index) * btnWidth + (btnWidth - 24) / 2
        }
    }

    @objc private func tabTapped(_ sender: NSButton) {
        selectTab(sender.tag)
        delegate?.tabBar(self, didSelectTab: sender.tag)
    }

    func refreshColors() {
        layer?.backgroundColor = BarkTheme.panelBackground.cgColor
        indicator.layer?.backgroundColor = BarkTheme.brand.cgColor
        for (i, btn) in buttons.enumerated() {
            btn.contentTintColor = i == selectedIndex ? BarkTheme.brand : BarkTheme.secondaryText
        }
    }
}
