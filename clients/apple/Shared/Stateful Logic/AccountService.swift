import Foundation
import SwiftLockbookCore

#if os(macOS)
import AppKit
import Combine
#endif

class AccountService: ObservableObject {
    let core: LockbookApi
    
    @Published var account: Account? = nil
    var calculated = false
        
    init(_ core: LockbookApi) {
        self.core = core
        switch core.getAccount() {
        case .success(let account):
            self.account = account
        case .failure(let error):
            switch error.kind {
            case .UiError(let getAccountError):
                switch getAccountError {
                case .NoAccount:
                    account = nil
                }
            case .Unexpected(_):
                DI.errors.handleError(error)
            }
        }
        
        calculated = true
    }
        
    func getAccount() {
        if account == nil {
            switch core.getAccount() {
            case .success(let account):
                self.account = account
            case .failure(let error):
                switch error.kind {
                case .UiError(let getAccountError):
                    switch getAccountError {
                    case .NoAccount:
                        print("account get unsuccessful")
                        self.account = nil
                    }
                case .Unexpected(_):
                    DI.errors.handleError(error)
                }
            }
        }
    }
    
    func logout() {
        DI.freshState()
        core.logoutAndExit()
    }
    
    func deleteAccount() {
        switch core.deleteAccount() {
        case .success(_):
            DI.freshState()
        case .failure(let error):
            DI.errors.handleError(error)
        }
    }
}
