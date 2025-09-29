import WidgetKit
import SwiftUI
import Intents

// MARK: - Widget Configuration
struct DNPStatusWidget: Widget {
    let kind: String = "DNPStatusWidget"

    var body: some WidgetConfiguration {
        StaticConfiguration(kind: kind, provider: DNPStatusProvider()) { entry in
            DNPStatusWidgetEntryView(entry: entry)
        }
        .configurationDisplayName("DNP Status")
        .description("Shows your Do Not Play list status and enforcement information")
        .supportedFamilies([.systemSmall, .systemMedium])
    }
}

// MARK: - Timeline Provider
struct DNPStatusProvider: TimelineProvider {
    func placeholder(in context: Context) -> DNPStatusEntry {
        DNPStatusEntry(
            date: Date(),
            dnpCount: 42,
            connectedServices: ["Spotify", "Apple Music"],
            lastEnforcement: "2 hours ago",
            isLoading: false
        )
    }

    func getSnapshot(in context: Context, completion: @escaping (DNPStatusEntry) -> ()) {
        let entry = DNPStatusEntry(
            date: Date(),
            dnpCount: 42,
            connectedServices: ["Spotify", "Apple Music"],
            lastEnforcement: "2 hours ago",
            isLoading: false
        )
        completion(entry)
    }

    func getTimeline(in context: Context, completion: @escaping (Timeline<DNPStatusEntry>) -> ()) {
        Task {
            do {
                let status = try await fetchEnforcementStatus()
                let entry = DNPStatusEntry(
                    date: Date(),
                    dnpCount: status.dnpCount,
                    connectedServices: status.connectedServices,
                    lastEnforcement: status.lastEnforcement,
                    isLoading: false
                )
                
                let nextUpdate = Calendar.current.date(byAdding: .minute, value: 15, to: Date())!
                let timeline = Timeline(entries: [entry], policy: .after(nextUpdate))
                completion(timeline)
            } catch {
                let errorEntry = DNPStatusEntry(
                    date: Date(),
                    dnpCount: 0,
                    connectedServices: [],
                    lastEnforcement: "Error loading",
                    isLoading: false
                )
                let timeline = Timeline(entries: [errorEntry], policy: .after(Date().addingTimeInterval(300)))
                completion(timeline)
            }
        }
    }
}

// MARK: - Timeline Entry
struct DNPStatusEntry: TimelineEntry {
    let date: Date
    let dnpCount: Int
    let connectedServices: [String]
    let lastEnforcement: String
    let isLoading: Bool
}

// MARK: - Widget View
struct DNPStatusWidgetEntryView: View {
    var entry: DNPStatusProvider.Entry

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Image(systemName: "minus.circle.fill")
                    .foregroundColor(.red)
                Text("DNP Status")
                    .font(.headline)
                    .fontWeight(.semibold)
                Spacer()
            }
            
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text("\(entry.dnpCount)")
                        .font(.title2)
                        .fontWeight(.bold)
                    Text("Artists Blocked")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                
                Spacer()
                
                VStack(alignment: .trailing, spacing: 4) {
                    Text("\(entry.connectedServices.count)")
                        .font(.title2)
                        .fontWeight(.bold)
                    Text("Services")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
            
            Spacer()
            
            HStack {
                Text("Last: \(entry.lastEnforcement)")
                    .font(.caption)
                    .foregroundColor(.secondary)
                Spacer()
                if entry.isLoading {
                    ProgressView()
                        .scaleEffect(0.8)
                }
            }
        }
        .padding()
        .background(Color(.systemBackground))
    }
}

// MARK: - API Service
struct EnforcementStatus {
    let dnpCount: Int
    let connectedServices: [String]
    let lastEnforcement: String
}

func fetchEnforcementStatus() async throws -> EnforcementStatus {
    guard let url = URL(string: "https://api.nodrakeinthe.house/v1/enforcement/status") else {
        throw URLError(.badURL)
    }
    
    var request = URLRequest(url: url)
    request.setValue("Bearer \(getAPIToken())", forHTTPHeaderField: "Authorization")
    
    let (data, _) = try await URLSession.shared.data(for: request)
    let response = try JSONDecoder().decode(EnforcementStatusResponse.self, from: data)
    
    return EnforcementStatus(
        dnpCount: response.dnpCount,
        connectedServices: response.connectedServices,
        lastEnforcement: response.lastEnforcement
    )
}

struct EnforcementStatusResponse: Codable {
    let dnpCount: Int
    let connectedServices: [String]
    let lastEnforcement: String
    
    enum CodingKeys: String, CodingKey {
        case dnpCount = "dnp_count"
        case connectedServices = "connected_services"
        case lastEnforcement = "last_enforcement"
    }
}

func getAPIToken() -> String {
    // In a real implementation, this would retrieve the token from Keychain
    return UserDefaults.standard.string(forKey: "api_token") ?? ""
}

// MARK: - Preview
struct DNPStatusWidget_Previews: PreviewProvider {
    static var previews: some View {
        DNPStatusWidgetEntryView(entry: DNPStatusEntry(
            date: Date(),
            dnpCount: 42,
            connectedServices: ["Spotify", "Apple Music"],
            lastEnforcement: "2 hours ago",
            isLoading: false
        ))
        .previewContext(WidgetPreviewContext(family: .systemSmall))
    }
}