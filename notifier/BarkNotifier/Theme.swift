import Cocoa

// MARK: - Theme Mode

enum ThemeMode: String {
    case light, dark, system

    var appearance: NSAppearance? {
        switch self {
        case .light:  return NSAppearance(named: .aqua)
        case .dark:   return NSAppearance(named: .darkAqua)
        case .system: return nil  // follows system
        }
    }

    /// Resolve to actual light/dark based on current system state.
    var isDark: Bool {
        switch self {
        case .dark: return true
        case .light: return false
        case .system:
            return NSApp.effectiveAppearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
        }
    }
}

// MARK: - BarkTheme

struct BarkTheme {

    static var mode: ThemeMode = {
        let raw = UserDefaults.standard.string(forKey: "barkThemeMode") ?? "dark"
        return ThemeMode(rawValue: raw) ?? .dark
    }() {
        didSet {
            UserDefaults.standard.set(mode.rawValue, forKey: "barkThemeMode")
            NotificationCenter.default.post(name: .barkThemeChanged, object: nil)
        }
    }

    private static var dark: Bool { mode.isDark }

    // MARK: - Panel Colors

    static var panelBackground: NSColor {
        dark ? NSColor(red: 0.12, green: 0.12, blue: 0.14, alpha: 1)
             : NSColor(red: 0.96, green: 0.96, blue: 0.97, alpha: 1)
    }
    static var cardBackground: NSColor {
        dark ? NSColor(red: 0.18, green: 0.18, blue: 0.20, alpha: 1)
             : NSColor.white
    }
    static var cardBorder: NSColor {
        dark ? NSColor(red: 0.25, green: 0.25, blue: 0.28, alpha: 1)
             : NSColor(red: 0.88, green: 0.88, blue: 0.90, alpha: 1)
    }
    static var separator: NSColor {
        dark ? NSColor(red: 0.22, green: 0.22, blue: 0.25, alpha: 1)
             : NSColor(red: 0.90, green: 0.90, blue: 0.92, alpha: 1)
    }

    // MARK: - Text Colors

    static var primaryText: NSColor {
        dark ? NSColor(red: 0.92, green: 0.92, blue: 0.94, alpha: 1)
             : NSColor(red: 0.11, green: 0.11, blue: 0.12, alpha: 1)
    }
    static var secondaryText: NSColor {
        dark ? NSColor(red: 0.65, green: 0.65, blue: 0.68, alpha: 1)
             : NSColor(red: 0.43, green: 0.43, blue: 0.45, alpha: 1)
    }
    static var dimText: NSColor {
        dark ? NSColor(red: 0.45, green: 0.45, blue: 0.48, alpha: 1)
             : NSColor(red: 0.60, green: 0.60, blue: 0.63, alpha: 1)
    }
    static var sectionHeader: NSColor {
        dark ? NSColor(red: 0.55, green: 0.55, blue: 0.58, alpha: 1)
             : NSColor(red: 0.50, green: 0.50, blue: 0.53, alpha: 1)
    }

    // MARK: - Accent Colors

    static let brand = NSColor(red: 0.25, green: 0.60, blue: 1.00, alpha: 1)
    static let running = NSColor.systemGreen
    static let stopped = NSColor.systemRed

    static func riskColor(_ level: String) -> NSColor {
        switch level.uppercased() {
        case "HIGH", "HIG", "HI": return .systemRed
        case "MEDIUM", "MED":     return .systemYellow
        case "LOW":               return .systemGreen
        default:                  return secondaryText
        }
    }

    static let accentAssessments = brand
    static let accentHighRisk    = NSColor.systemRed
    static let accentAllowed     = NSColor.systemGreen
    static let accentDenied      = NSColor.systemOrange

    // MARK: - Fonts

    static let headerTitle     = NSFont.systemFont(ofSize: 14, weight: .semibold)
    static let statusLabel     = NSFont.systemFont(ofSize: 11, weight: .medium)
    static let statValue       = NSFont.monospacedDigitSystemFont(ofSize: 28, weight: .bold)
    static let statLabel       = NSFont.systemFont(ofSize: 10, weight: .medium)
    static let sectionTitle    = NSFont.systemFont(ofSize: 11, weight: .semibold)
    static let logRiskBadge    = NSFont.monospacedSystemFont(ofSize: 9, weight: .bold)
    static let logDescription  = NSFont.monospacedSystemFont(ofSize: 11, weight: .regular)
    static let logTimestamp    = NSFont.monospacedDigitSystemFont(ofSize: 10, weight: .regular)
    static let logSource       = NSFont.monospacedSystemFont(ofSize: 9, weight: .medium)
    static let body            = NSFont.systemFont(ofSize: 12, weight: .regular)
    static let bodyBold        = NSFont.systemFont(ofSize: 12, weight: .semibold)
    static let small           = NSFont.systemFont(ofSize: 11, weight: .regular)
    static let buttonFont      = NSFont.systemFont(ofSize: 12, weight: .medium)

    // MARK: - Layout

    static let popoverWidth:  CGFloat = 360
    static let popoverHeight: CGFloat = 480
    static let padding:       CGFloat = 14
    static let cardSpacing:   CGFloat = 8
    static let rowHeight:     CGFloat = 28
    static let cardHeight:    CGFloat = 72
    static let cornerRadius:  CGFloat = 8
    static let tabBarHeight:  CGFloat = 40
}

// MARK: - Notification

extension Notification.Name {
    static let barkThemeChanged = Notification.Name("barkThemeChanged")
    static let barkHookStateChanged = Notification.Name("barkHookStateChanged")
}

// MARK: - Helpers

func makeLabel(_ text: String, font: NSFont, color: NSColor, frame: NSRect) -> NSTextField {
    let label = NSTextField(labelWithString: text)
    label.font = font
    label.textColor = color
    label.frame = frame
    return label
}

/// Flipped NSView — origin at top-left, matching natural top-down layout.
/// Use as NSScrollView.documentView so content starts at the top.
class FlippedView: NSView {
    override var isFlipped: Bool { true }
}

func makeSeparator(y: CGFloat, width: CGFloat) -> NSView {
    let sep = NSView(frame: NSRect(x: 0, y: y, width: width, height: 1))
    sep.wantsLayer = true
    sep.layer?.backgroundColor = BarkTheme.separator.cgColor
    return sep
}
