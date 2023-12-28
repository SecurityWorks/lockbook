import Foundation
import SwiftUI
import MetalKit
import Combine

#if os(iOS)
public struct WorkspaceView: UIViewRepresentable {
    
    @ObservedObject public var workspaceState: WorkspaceState
    let mtkView: iOSMTK = iOSMTK()
    
    @Environment(\.horizontalSizeClass) var horizontal
    @Environment(\.verticalSizeClass) var vertical
    
    public init(_ workspaceState: WorkspaceState, _ coreHandle: UnsafeMutableRawPointer?, _ toolbarState: ToolbarState, _ nameState: NameState) {
        self.workspaceState = workspaceState
        mtkView.workspaceState = workspaceState
        mtkView.toolbarState = toolbarState
        mtkView.nameState = nameState
        
        mtkView.setInitialContent(coreHandle, workspaceState.text)
    }

    public func makeUIView(context: Context) -> iOSMTK {
        return mtkView
    }
    
    public func updateUIView(_ uiView: iOSMTK, context: Context) {
        if workspaceState.shouldFocus {
            mtkView.becomeFirstResponder()
            workspaceState.shouldFocus = false
        }
    }
}
#else
public struct WorkspaceView: View, Equatable {
    @FocusState var focused: Bool
    @ObservedObject var workspaceState: WorkspaceState
    
    let nsEditorView: NSWS
    
    public init(_ workspaceState: WorkspaceState, _ coreHandle: UnsafeMutableRawPointer?) {
        self.workspaceState = workspaceState
        nsEditorView = NSWS(workspaceState, coreHandle)
    }
    
    public var body: some View {
        nsEditorView
            .focused($focused)
            .onAppear {
                focused = true
            }
            .onChange(of: workspaceState.shouldFocus, perform: { newValue in
                if newValue {
                    focused = true
                }
            })

    }
    
    public static func == (lhs: WorkspaceView, rhs: WorkspaceView) -> Bool {
        true
    }
}

public struct NSWS: NSViewRepresentable {
    
    @ObservedObject public var workspaceState: WorkspaceState
    let coreHandle: UnsafeMutableRawPointer?
    let mtkView: MacMTK = MacMTK()
    
    public init(_ workspaceState: WorkspaceState, _ coreHandle: UnsafeMutableRawPointer?) {
        self.workspaceState = workspaceState
        mtkView.workspaceState = workspaceState
        self.coreHandle = coreHandle
        
    }
    
    public func makeNSView(context: NSViewRepresentableContext<NSWS>) -> MTKView {
        mtkView.setInitialContent(coreHandle)
        return mtkView
    }
    
    public func updateNSView(_ nsView: MTKView, context: NSViewRepresentableContext<NSWS>) {
        if let id = workspaceState.openDoc {
            print(id)
            mtkView.openFile(id: id)
        }
        
        if workspaceState.shouldFocus {
            workspaceState.shouldFocus = false
        }
    }
}
#endif




