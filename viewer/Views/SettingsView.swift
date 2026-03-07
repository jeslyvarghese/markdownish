import SwiftUI

struct SettingsView: View {
    @Environment(AppState.self) private var appState

    var body: some View {
        @Bindable var state = appState

        ScrollView {
            VStack(alignment: .leading, spacing: 24) {
                SectionHeader("Appearance")

                VStack(alignment: .leading, spacing: 12) {
                    HStack {
                        Text("Theme")
                            .font(.system(size: 12))
                        Spacer()
                        Picker("", selection: $state.theme) {
                            Text("Dark").tag(Theme.dark)
                            Text("Light").tag(Theme.light)
                        }
                        .pickerStyle(.segmented)
                        .frame(width: 120)
                    }
                }

                SectionHeader("Typography")

                VStack(alignment: .leading, spacing: 12) {
                    LabeledSlider(
                        label: "Font Size",
                        value: $state.fontSize,
                        range: 12...24,
                        step: 1,
                        format: { "\(Int($0))pt" }
                    )
                }

                SectionHeader("Layout")

                VStack(alignment: .leading, spacing: 12) {
                    HStack {
                        Text("Width")
                            .font(.system(size: 12))
                        Spacer()
                        Picker("", selection: $state.contentWidth) {
                            ForEach(ContentWidthMode.allCases, id: \.self) { mode in
                                Text(mode.rawValue).tag(mode)
                            }
                        }
                        .pickerStyle(.menu)
                        .frame(width: 120)
                    }
                }

                Spacer()
            }
            .padding(16)
        }
        .background(.ultraThinMaterial)
    }
}

// MARK: - Components

struct SectionHeader: View {
    let title: String
    init(_ title: String) { self.title = title }

    var body: some View {
        Text(title)
            .font(.system(size: 10, weight: .semibold))
            .foregroundStyle(.secondary)
            .textCase(.uppercase)
            .tracking(0.8)
    }
}

struct LabeledSlider: View {
    let label: String
    @Binding var value: CGFloat
    let range: ClosedRange<CGFloat>
    let step: CGFloat
    let format: (CGFloat) -> String

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(label).font(.system(size: 12))
                Spacer()
                Text(format(value))
                    .font(.system(size: 11, design: .monospaced))
                    .foregroundStyle(.secondary)
            }
            Slider(value: $value, in: range, step: step)
        }
    }
}
