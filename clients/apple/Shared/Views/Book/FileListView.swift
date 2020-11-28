import SwiftUI
import SwiftLockbookCore

struct FileListView: View {
    @ObservedObject var core: Core
    let account: Account
    let root: FileMetadata
    @State var creating: FileType?
    @State var creatingName: String = ""
    @State var path: [FileMetadata]
    @State var currentFolder: FileMetadata

    var body: some View {
        let filtered = computeFileList()
        let baseView = List {
            Button(action: handleNavigateUp) {
                HStack {
                    Image(systemName: "arrow.turn.left.up")
                        .foregroundColor(.accentColor)
                    ForEach(path) {
                        Text($0.name)
                        Text("/")
                            .foregroundColor(.accentColor)
                    }
                    Text(currentFolder.name)
                }
            }
            creating.map { creatingType in
                SyntheticFileCell(params: (currentFolder, creatingType), nameField: $creatingName, onCreate: {
                    handleCreate(meta: currentFolder, type: creatingType)
                }, onCancel: doneCreating)
            }
            ForEach(filtered) { meta in
                renderCell(meta: meta)
                    .contextMenu(menuItems: {
                        Button(action: {
                            handleDelete(meta: meta)
                        }) {
                            Label("Delete", systemImage: "trash.fill")
                        }
                    })
            }
            .onDelete(perform: {
                handleDelete(meta: filtered[$0.first!])
            })
        }
        .onReceive(core.timer, perform: { _ in
            core.sync()
        })

        #if os(iOS)
        return baseView
            .toolbar {
                ToolbarItemGroup(placement: .bottomBar) {
                    Button(action: { creating = .Folder }) {
                        Image(systemName: "folder.fill.badge.plus")
                    }
                    Button(action: { creating = .Document }) {
                        Image(systemName: "doc.on.doc.fill")
                    }
                    Spacer()
                    Spacer()
                    Text("\(core.files.count) items")
                        .foregroundColor(.secondary)
                    Spacer()
                    Spacer()
                    Spacer()
                    ProgressView()
                        .opacity(core.syncing ? 1.0 : 0)
                }
            }
        #else
        return VStack {
            baseView
                .toolbar {
                    ToolbarItemGroup(placement: .primaryAction, content: { HStack { } })
                }
            Spacer()
            HStack {
                Button(action: { creating = .Folder }) {
                    Image(systemName: "folder.badge.plus")
                }
                Button(action: { creating = .Document }) {
                    Image(systemName: "doc.on.doc")
                }
                Spacer()
                Text("\(core.files.count) items")
                    .foregroundColor(.secondary)
                Spacer()
                ProgressView()
                    .controlSize(.small)
                    .opacity(core.syncing ? 1.0 : 0)
            }
            .padding()
        }
        #endif
    }

    init(core: Core, account: Account, root: FileMetadata) {
        self.core = core
        self.account = account
        self.root = root
        self._path = .init(initialValue: [])
        self._currentFolder = .init(initialValue: root)
    }

    func handleCreate(meta: FileMetadata, type: FileType) {
        switch core.api.createFile(name: creatingName, dirId: meta.id, isFolder: type == .Folder) {
        case .success(_):
            doneCreating()
            core.updateFiles()
        case .failure(let err):
            core.handleError(err)
        }
    }

    func handleDelete(meta: FileMetadata) {
        switch core.api.deleteFile(id: meta.id) {
        case .success(_):
            core.updateFiles()
        case .failure(let err):
            core.handleError(err)
        }
    }

    func handleSelectFolder(meta: FileMetadata) {
        withAnimation {
            path.append(currentFolder)
            currentFolder = meta
        }
    }

    func handleNavigateUp() {
        withAnimation {
            path.popLast().map {
                currentFolder = $0
            }
        }
    }

    func computeFileList() -> [FileMetadata] {
        core.files.filter {
            $0.parent == currentFolder.id && $0.id != currentFolder.id
        }
    }

    func doneCreating() {
        withAnimation {
            creating = .none
            creatingName = ""
        }
    }

    func renderCell(meta: FileMetadata) -> AnyView {
        if meta.fileType == .Folder {
            return AnyView(
                Button(action: { handleSelectFolder(meta: meta) }) {
                    FileCell(meta: meta)
                }
            )
        } else {
            return AnyView(
                NavigationLink(destination: EditorView(core: core, meta: meta).equatable()) {
                    FileCell(meta: meta)
                }
            )
        }
    }
}

struct FileListView_Previews: PreviewProvider {
    static let core = Core()

    static var previews: some View {
        NavigationView {
            FileListView(core: core, account: core.account!, root: core.root!)
        }
    }
}