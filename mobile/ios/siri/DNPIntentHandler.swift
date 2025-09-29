import Intents
import Foundation

// MARK: - Intent Definitions
@available(iOS 12.0, *)
class DNPIntentHandler: INExtension, AddArtistToDNPIntentHandling, RemoveArtistFromDNPIntentHandling, CheckEnforcementStatusIntentHandling {
    
    override func handler(for intent: INIntent) -> Any {
        return self
    }
    
    // MARK: - Add Artist to DNP Intent
    func handle(intent: AddArtistToDNPIntent, completion: @escaping (AddArtistToDNPIntentResponse) -> Void) {
        guard let artistName = intent.artistName, !artistName.isEmpty else {
            completion(AddArtistToDNPIntentResponse(code: .failure, userActivity: nil))
            return
        }
        
        Task {
            do {
                let artist = try await searchAndAddArtist(name: artistName)
                let response = AddArtistToDNPIntentResponse(code: .success, userActivity: nil)
                response.artistName = artist.name
                completion(response)
            } catch {
                let response = AddArtistToDNPIntentResponse(code: .failure, userActivity: nil)
                response.errorMessage = error.localizedDescription
                completion(response)
            }
        }
    }
    
    func resolveArtistName(for intent: AddArtistToDNPIntent, with completion: @escaping (INStringResolutionResult) -> Void) {
        guard let artistName = intent.artistName, !artistName.isEmpty else {
            completion(INStringResolutionResult.needsValue())
            return
        }
        completion(INStringResolutionResult.success(with: artistName))
    }
    
    // MARK: - Remove Artist from DNP Intent
    func handle(intent: RemoveArtistFromDNPIntent, completion: @escaping (RemoveArtistFromDNPIntentResponse) -> Void) {
        guard let artistName = intent.artistName, !artistName.isEmpty else {
            completion(RemoveArtistFromDNPIntentResponse(code: .failure, userActivity: nil))
            return
        }
        
        Task {
            do {
                try await removeArtistFromDNP(name: artistName)
                let response = RemoveArtistFromDNPIntentResponse(code: .success, userActivity: nil)
                response.artistName = artistName
                completion(response)
            } catch {
                let response = RemoveArtistFromDNPIntentResponse(code: .failure, userActivity: nil)
                response.errorMessage = error.localizedDescription
                completion(response)
            }
        }
    }
    
    func resolveArtistName(for intent: RemoveArtistFromDNPIntent, with completion: @escaping (INStringResolutionResult) -> Void) {
        guard let artistName = intent.artistName, !artistName.isEmpty else {
            completion(INStringResolutionResult.needsValue())
            return
        }
        completion(INStringResolutionResult.success(with: artistName))
    }
    
    // MARK: - Check Enforcement Status Intent
    func handle(intent: CheckEnforcementStatusIntent, completion: @escaping (CheckEnforcementStatusIntentResponse) -> Void) {
        Task {
            do {
                let status = try await fetchEnforcementStatus()
                let response = CheckEnforcementStatusIntentResponse(code: .success, userActivity: nil)
                response.dnpCount = NSNumber(value: status.dnpCount)
                response.connectedServices = status.connectedServices
                response.lastEnforcement = status.lastEnforcement
                completion(response)
            } catch {
                let response = CheckEnforcementStatusIntentResponse(code: .failure, userActivity: nil)
                response.errorMessage = error.localizedDescription
                completion(response)
            }
        }
    }
}

// MARK: - API Service Functions
extension DNPIntentHandler {
    
    private func searchAndAddArtist(name: String) async throws -> Artist {
        // Search for artist
        let searchResults = try await searchArtist(name: name)
        guard let artist = searchResults.first else {
            throw DNPError.artistNotFound
        }
        
        // Add to DNP list
        try await addArtistToDNP(artistId: artist.id)
        return artist
    }
    
    private func searchArtist(name: String) async throws -> [Artist] {
        guard let url = URL(string: "https://api.nodrakeinthe.house/v1/artists/search") else {
            throw DNPError.invalidURL
        }
        
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("Bearer \(getAPIToken())", forHTTPHeaderField: "Authorization")
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        
        let searchRequest = ArtistSearchRequest(query: name, limit: 5)
        request.httpBody = try JSONEncoder().encode(searchRequest)
        
        let (data, _) = try await URLSession.shared.data(for: request)
        let response = try JSONDecoder().decode(ArtistSearchResponse.self, from: data)
        return response.artists
    }
    
    private func addArtistToDNP(artistId: String) async throws {
        guard let url = URL(string: "https://api.nodrakeinthe.house/v1/dnp/artists") else {
            throw DNPError.invalidURL
        }
        
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("Bearer \(getAPIToken())", forHTTPHeaderField: "Authorization")
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        
        let addRequest = AddArtistRequest(
            artistId: artistId,
            tags: ["siri-added"],
            note: "Added via Siri"
        )
        request.httpBody = try JSONEncoder().encode(addRequest)
        
        let (_, response) = try await URLSession.shared.data(for: request)
        guard let httpResponse = response as? HTTPURLResponse,
              httpResponse.statusCode == 200 else {
            throw DNPError.apiError
        }
    }
    
    private func removeArtistFromDNP(name: String) async throws {
        // First get the DNP list to find the artist
        let dnpList = try await fetchDNPList()
        guard let artist = dnpList.first(where: { $0.name.lowercased().contains(name.lowercased()) }) else {
            throw DNPError.artistNotInDNP
        }
        
        guard let url = URL(string: "https://api.nodrakeinthe.house/v1/dnp/artists/\(artist.id)") else {
            throw DNPError.invalidURL
        }
        
        var request = URLRequest(url: url)
        request.httpMethod = "DELETE"
        request.setValue("Bearer \(getAPIToken())", forHTTPHeaderField: "Authorization")
        
        let (_, response) = try await URLSession.shared.data(for: request)
        guard let httpResponse = response as? HTTPURLResponse,
              httpResponse.statusCode == 200 else {
            throw DNPError.apiError
        }
    }
    
    private func fetchDNPList() async throws -> [Artist] {
        guard let url = URL(string: "https://api.nodrakeinthe.house/v1/dnp/artists") else {
            throw DNPError.invalidURL
        }
        
        var request = URLRequest(url: url)
        request.setValue("Bearer \(getAPIToken())", forHTTPHeaderField: "Authorization")
        
        let (data, _) = try await URLSession.shared.data(for: request)
        let response = try JSONDecoder().decode(DNPListResponse.self, from: data)
        return response.artists
    }
    
    private func getAPIToken() -> String {
        // In a real implementation, this would retrieve the token from Keychain
        return UserDefaults.standard.string(forKey: "api_token") ?? ""
    }
}

// MARK: - Data Models
struct Artist: Codable {
    let id: String
    let name: String
    let externalIds: [String: String]?
    
    enum CodingKeys: String, CodingKey {
        case id, name
        case externalIds = "external_ids"
    }
}

struct ArtistSearchRequest: Codable {
    let query: String
    let limit: Int
}

struct ArtistSearchResponse: Codable {
    let artists: [Artist]
}

struct AddArtistRequest: Codable {
    let artistId: String
    let tags: [String]
    let note: String
    
    enum CodingKeys: String, CodingKey {
        case artistId = "artist_id"
        case tags, note
    }
}

struct DNPListResponse: Codable {
    let artists: [Artist]
}

// MARK: - Error Types
enum DNPError: LocalizedError {
    case invalidURL
    case artistNotFound
    case artistNotInDNP
    case apiError
    
    var errorDescription: String? {
        switch self {
        case .invalidURL:
            return "Invalid API URL"
        case .artistNotFound:
            return "Artist not found"
        case .artistNotInDNP:
            return "Artist not in DNP list"
        case .apiError:
            return "API request failed"
        }
    }
}