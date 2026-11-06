import SwiftUI

struct BatteryCardView: View, Equatable {
    let battery: BatteryStatus
    var compact = false

    var body: some View {
        CardView {
            VStack(alignment: compact ? .leading : .center, spacing: 10) {
                HStack {
                    Label("电池", systemImage: batteryIcon(battery.capacity))
                        .font(.subheadline.weight(.semibold))
                    if compact {
                        Spacer(minLength: 4)
                        Image(systemName: "chevron.right").font(.caption2).foregroundStyle(.tertiary)
                    }
                }
                // A battery percentage must render as one exact value, not independent reels.
                Text(battery.capacityText)
                    .font(.title3.weight(.bold).monospacedDigit())
                    .transaction { $0.animation = nil }
                batteryStatusLine
                if !compact {
                    TemperatureValueText(value: battery.temperature)
                        .font(.caption).foregroundStyle(.secondary)
                } else {
                    Text("电源详情").font(.caption).foregroundStyle(.secondary)
                }
            }
            .frame(maxWidth: .infinity, alignment: compact ? .leading : .center)
        }
    }

    @ViewBuilder
    private var batteryStatusLine: some View {
        HStack(spacing: 4) {
            batteryStatusText
            if let ma = battery.currentMA {
                Text("\u{00B7}").font(.caption).foregroundStyle(.secondary)
                if let mv = battery.voltageMV {
                    let watts = Double(mv) * Double(abs(ma)) / 1_000_000.0
                    AnimatedNumber(value: watts, decimalPlaces: 1,
                                   font: .caption.monospacedDigit(),
                                   textColor: batteryStatusColor,
                                   suffix: "W")
                } else {
                    AnimatedNumber(value: ma,
                                   font: .caption.monospacedDigit(),
                                   textColor: batteryStatusColor,
                                   prefix: ma >= 0 ? "+" : nil,
                                   suffix: "mA")
                }
            }
        }
        .lineLimit(1)
        .minimumScaleFactor(0.8)
    }

    @ViewBuilder
    private var batteryStatusText: some View {
        switch battery.charging {
        case "stopped":
            Text("已停止充电")
                .font(.caption)
                .foregroundStyle(.orange)
        case "charging":
            if battery.capacityPercent == 100 {
                Text("已充满")
                    .font(.caption)
                    .foregroundStyle(.green)
            } else if battery.currentMA != nil {
                Text("充电中")
                    .font(.caption)
                    .foregroundStyle(.green)
            } else if battery.timeToFull > 0 {
                Text("充电（\u{00B7} \(formatETA(battery.timeToFull)）")
                    .font(.caption.monospacedDigit())
                    .foregroundStyle(.green)
            } else {
                Text("充电中")
                    .font(.caption)
                    .foregroundStyle(.green)
            }
        case "":
            Text("不可用").font(.caption).foregroundStyle(.secondary)
        default:
            if battery.timeToEmpty > 0 {
                if battery.currentMA != nil {
                    Text(formatETA(battery.timeToEmpty))
                        .font(.caption.monospacedDigit())
                        .foregroundStyle(Color.batteryColor(battery.capacity))
                } else {
                    Text("\(formatETA(battery.timeToEmpty)) left")
                        .font(.caption.monospacedDigit())
                        .foregroundStyle(Color.batteryColor(battery.capacity))
                }
            } else {
                Text("放电中")
                    .font(.caption)
                    .foregroundStyle(Color.batteryColor(battery.capacity))
            }
        }
    }

    private var batteryStatusColor: Color {
        switch battery.charging {
        case "stopped": return .orange
        case "charging": return .green
        default: return Color.batteryColor(battery.capacity)
        }
    }

    private func formatETA(_ minutes: Int) -> String {
        if minutes >= 1440 {
            let d = minutes / 1440, h = (minutes % 1440) / 60, m = minutes % 60
            return "\(d)d \(h)h \(m)m"
        }
        return minutes >= 60 ? "\(minutes / 60)h \(minutes % 60)m" : "\(minutes)m"
    }

    private func batteryIcon(_ percent: Int) -> String {
        guard (0...100).contains(percent) else { return "battery.0" }
        if percent >= 75 { return "battery.100" }
        if percent >= 50 { return "battery.75" }
        if percent >= 25 { return "battery.50" }
        return "battery.25"
    }
}

struct BatteryDetailSheet: View {
    let battery: BatteryStatus
    @Environment(\.dismiss) private var dismiss
    @Environment(\.accessibilityReduceMotion) private var reduceMotion

    var body: some View {
        NavigationStack {
            List {
                row("容量", icon: batteryIcon(battery.capacity), value: battery.capacityText)
                row("状态", icon: "bolt.fill", value: statusLabel)
                if let mv = battery.voltageMV {
                    numericRow("电压", icon: "bolt.circle", value: Double(mv) / 1000, decimals: 3, unit: " V")
                }
                if let ma = battery.currentMA {
                    numericRow("当前", icon: "arrow.left.arrow.right", value: Double(ma), decimals: 0, unit: " mA")
                }
                if let mv = battery.voltageMV, let ma = battery.currentMA {
                    let watts = Double(mv) * Double(abs(ma)) / 1_000_000.0
                    numericRow("电源", icon: "flame", value: watts, decimals: 1, unit: " W")
                }
                HStack {
                    Label("温度", systemImage: "thermometer.medium")
                    Spacer()
                    TemperatureValueText(value: battery.temperature).foregroundStyle(.secondary)
                }
                row("满时间", icon: "battery.100.bolt", value: battery.charging == "charging" && battery.timeToFull > 0 ? formatETA(battery.timeToFull) : "—")
                row("清空时间", icon: "battery.25", value: battery.charging == "discharging" && battery.timeToEmpty > 0 ? formatETA(battery.timeToEmpty) : "—")
            }
            .toolbar { ToolbarItem(placement: .confirmationAction) { Button("完成") { dismiss() } } }
            .navigationTitle("电池详情")
            .navigationBarTitleDisplayMode(.inline)
        }
    }

    private func numericRow(_ label: String, icon: String, value: Double, decimals: Int, unit: String) -> some View {
        HStack {
            Label(label, systemImage: icon)
            Spacer()
            AnimatedNumber(value: value, decimalPlaces: decimals, font: .body,
                textColor: .secondary, animationDuration: reduceMotion ? 0 : 0.3, suffix: unit)
        }
    }

    private func row(_ label: String, icon: String, value: String) -> some View {
        HStack {
            Label(label, systemImage: icon)
            Spacer()
            Text(value)
                .foregroundStyle(.secondary)
                .monospacedDigit()
        }
    }

    private var statusLabel: String {
        switch battery.charging {
        case "stopped": return "已停止充电"
        case "charging": return battery.capacityPercent == 100 ? "已充满" : "充电中"
        case "": return "不可用"
        default: return "放电中"
        }
    }

    private func formatETA(_ minutes: Int) -> String {
        if minutes >= 1440 {
            let d = minutes / 1440, h = (minutes % 1440) / 60, m = minutes % 60
            return "\(d)d \(h)h \(m)m"
        }
        return minutes >= 60 ? "\(minutes / 60)h \(minutes % 60)m" : "\(minutes)m"
    }

    private func batteryIcon(_ percent: Int) -> String {
        guard (0...100).contains(percent) else { return "battery.0" }
        if percent >= 75 { return "battery.100" }
        if percent >= 50 { return "battery.75" }
        if percent >= 25 { return "battery.50" }
        return "battery.25"
    }
}
