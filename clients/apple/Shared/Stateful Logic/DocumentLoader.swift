import Foundation
import SwiftLockbookCore
import Combine
import PencilKit
import SwiftUI

public enum ViewType {
    case Markdown
    #if os(iOS)
    case Drawing
    #endif
    case Image
    case Unknown
}

class DocumentLoader: ObservableObject {
    let core: LockbookApi
    
    @Published var meta: ClientFileMetadata?
    @Published var type: ViewType?
    @Published var deleted: Bool = false
    @Published var loading: Bool = true
    @Published var reloadContent: Bool = false
    
    @Published var textDocument: String?
    @Published var drawing: PKDrawing?
    @Published var image: Image? = .none
    
    private var cancellables = Set<AnyCancellable>()
    
    init(_ core: LockbookApi) {
        self.core = core
        setupAutoSavers()
    }
    
    func startLoading(_ meta: ClientFileMetadata) {
        print("loading \(meta.name)")
        let type = DocumentLoader.getType(name: meta.name)
        
        self.meta = meta
        self.type = type
        self.deleted = false
        self.loading = true
        self.reloadContent = false
        self.textDocument = nil
        self.drawing = nil
        self.image = nil
        
        if self.type == .Unknown {
            self.loading = false
            return
        }
        
        switch type {
        case .Markdown:
            loadText()
        #if os(iOS)
        case .Drawing:
            loadDrawing()
        #endif
        case .Image:
            loadImage()
        case .Unknown:
            self.loading = false
        }
    }
    
    func updatesFromCoreAvailable(_ newMeta: ClientFileMetadata) {
        self.meta = newMeta
        if
            let type = self.type,
            let meta = self.meta
        {
            switch type {
            case .Markdown: // For markdown we're able to do a check before reloading the doc
                DispatchQueue.global(qos: .userInitiated).async {
                    let operation = self.core.getFile(id: meta.id)
                    
                    DispatchQueue.main.async {
                        switch operation {
                        case .success(let txt):
                            if txt != self.textDocument {
                                self.reloadContent = true
                                self.textDocument = txt
                            }
                        case .failure(let err):
                            print(err)
                        }
                    }
                }
            #if os(iOS)
            case .Drawing:
                self.reloadContent = true
                loadDrawing()
            #endif
            case .Image:
                self.reloadContent = true
                loadImage()
            case .Unknown:
                print("cannot reload unknown content type")
            }
        } else {
            print("should not be reached")
        }
        
    }
    
    private func setupAutoSavers() {
        print("autosaver setup")
        $textDocument
            .debounce(for: .seconds(1), scheduler: DispatchQueue.global(qos: .userInitiated))
            .sink(receiveValue: {
                if let text = $0 { // TODO don't write if a reload or delete is required
                    self.writeDocument(content: text)
                }
            })
            .store(in: &cancellables)
        
        $drawing
            .debounce(for: .seconds(1), scheduler: DispatchQueue.global(qos: .userInitiated))
            .sink(receiveValue: {
                if let text = $0 { // TODO don't write if a reload or delete is required
                    self.writeDrawing(drawing: text)
                }
            })
            .store(in: &cancellables)
    }
    
    private func loadText() {
        if let meta = self.meta {
            
            DispatchQueue.global(qos: .userInitiated).async {
                let operation = self.core.getFile(id: meta.id)
                
                DispatchQueue.main.async {
                    switch operation {
                    case .success(let txt):
                        self.textDocument = txt
                        self.loading = false
                    case .failure(let err):
                        print(err)
                    }
                }
            }
        }
        
    }
    
    private func writeDocument(content: String) {
        if let meta = self.meta {
            print("write called: \(meta.name)")
            let operation = self.core.updateFile(id: meta.id, content: content)
            DispatchQueue.main.async {
                switch operation {
                case .success(_):
                    DI.sync.documentChangeHappened()
                case .failure(let error):
                    DI.errors.handleError(error)
                }
            }
        }
    }
    
    private func loadImage() {
        if let meta = self.meta {
            
            DispatchQueue.global(qos: .userInitiated).async {
                let operation = self.core.exportDrawing(id: meta.id)
                
                DispatchQueue.main.async {
                    switch operation {
                    case .success(let data):
                        if let image = self.getImage(from: data) {
                            self.image = image
                            self.loading = false
                        } else {
                            print("Could not make NSImage from Data!")
                        }
                    case .failure(let error):
                        DI.errors.handleError(error)
                    }
                }
            }
        }
    }
    
    private func getImage(from: Data) -> Image? {
        #if os(macOS)
        if let nsImage = NSImage(data: from) {
            return Image(nsImage: nsImage)
        } else {
            return .none
        }
        #else
        if let uiImage = UIImage(data: from) {
            return Image(uiImage: uiImage)
        } else {
            return .none
        }
        #endif
    }
    
    private func loadDrawing() {
        if let meta = self.meta {
            DispatchQueue.global(qos: .userInitiated).async {
                let operation = self.core.readDrawing(id: meta.id)
                DispatchQueue.main.async {
                    switch operation {
                    case .success(let drawing):
                        self.drawing = PKDrawing(from: drawing)
                        self.loading = false
                    case .failure(let error):
                        DI.errors.handleError(error)
                    }
                }
            }
        }
        
    }
    
    private func writeDrawing(drawing: PKDrawing) {
        if let meta = self.meta {
            
            switch self.core.writeDrawing(id: meta.id, content: Drawing(from: drawing)) {
            case .success(_):
                print("drawing saved successfully")
            case .failure(let error):
                DI.errors.handleError(error)
            }
            
            DI.sync.documentChangeHappened()
        }
    }
    
    // TODO we need the swift clients to accept Data back as files, then we can read arbitary images
    private static func getType(name: String) -> ViewType {
        if name.lowercased().hasSuffix(".draw") {
            #if os(macOS)
            return .Image
            #else
            return .Drawing
            #endif
        } else if name.lowercased().hasSuffix(".md") || name.lowercased().hasSuffix(".markdown") {
            return .Markdown
        } else {
            return .Unknown
        }
    }
}