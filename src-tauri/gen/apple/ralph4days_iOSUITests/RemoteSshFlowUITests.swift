import XCTest

final class RemoteSshFlowUITests: XCTestCase {
  override func setUpWithError() throws {
    continueAfterFailure = false
  }

  private func waitForExists(_ element: XCUIElement, _ message: String, timeout: TimeInterval = 30) {
    XCTAssertTrue(element.waitForExistence(timeout: timeout), message)
  }

  @discardableResult
  private func capture(_ app: XCUIApplication, _ name: String) -> String {
    let screenshot = XCUIScreen.main.screenshot()
    let attachment = XCTAttachment(screenshot: screenshot)
    attachment.name = name
    attachment.lifetime = .keepAlways
    add(attachment)

    let env = ProcessInfo.processInfo.environment
    guard let dir = env["RALPH_IOS_E2E_SCREENSHOT_DIR"], !dir.isEmpty else {
      return ""
    }

    let url = URL(fileURLWithPath: dir, isDirectory: true).appendingPathComponent("\(name).png")
    do {
      try FileManager.default.createDirectory(
        at: URL(fileURLWithPath: dir, isDirectory: true),
        withIntermediateDirectories: true
      )
      try screenshot.pngRepresentation.write(to: url, options: [.atomic])
      return url.path
    } catch {
      XCTFail("Failed to write screenshot '\(name)' to \(url.path): \(error)")
      return ""
    }
  }

  func testRemoteSshCrudAndDialogFlow() throws {
    let app = XCUIApplication()
    app.launchEnvironment["RUST_BACKTRACE"] = "full"
    app.launchEnvironment["RUST_LOG"] = "info"
    app.launch()

    let title = app.staticTexts["SSH Connections"]
    waitForExists(title, "Expected SSH Connections title")
    capture(app, "remote-ssh-00-empty")

    let newProfileButton = app.buttons["New Profile"]
    waitForExists(newProfileButton, "Expected New Profile button")
    newProfileButton.tap()

    let profileNameField = app.textFields["Work Mac"]
    waitForExists(profileNameField, "Expected profile name field")
    profileNameField.tap()
    profileNameField.typeText("E2E Host")

    let hostField = app.textFields["dev.example.com"]
    waitForExists(hostField, "Expected SSH host field")
    hostField.tap()
    hostField.typeText("127.0.0.1")

    let usernameField = app.textFields["vince"]
    waitForExists(usernameField, "Expected SSH username field")
    usernameField.tap()
    usernameField.typeText("ralph")

    capture(app, "remote-ssh-01-editor-filled")

    let saveButton = app.buttons["Save Profile"]
    waitForExists(saveButton, "Expected Save Profile button")
    saveButton.tap()

    let profileCardTitle = app.staticTexts["E2E Host"]
    waitForExists(profileCardTitle, "Expected created profile row")
    capture(app, "remote-ssh-02-profile-created")

    let connectButtons = app.buttons.matching(NSPredicate(format: "label == 'Connect'"))
    XCTAssertGreaterThan(connectButtons.count, 0, "Expected at least one Connect button")
    connectButtons.firstMatch.tap()

    let connectNowButton = app.buttons["Connect Now"]
    waitForExists(connectNowButton, "Expected Connect dialog to open")
    capture(app, "remote-ssh-03-connect-dialog")

    let cancelButtons = app.buttons.matching(NSPredicate(format: "label == 'Cancel'"))
    XCTAssertGreaterThan(cancelButtons.count, 0, "Expected cancel button")
    cancelButtons.firstMatch.tap()

    let deleteButton = app.buttons.matching(NSPredicate(format: "label == 'Delete'")).firstMatch
    waitForExists(deleteButton, "Expected Delete button on profile card")
    deleteButton.tap()

    let deleteButtons = app.buttons.matching(NSPredicate(format: "label == 'Delete'"))
    XCTAssertGreaterThan(deleteButtons.count, 0, "Expected delete confirmation dialog")
    let confirmDeleteButton = deleteButtons.element(boundBy: deleteButtons.count - 1)
    waitForExists(confirmDeleteButton, "Expected delete confirmation dialog")
    capture(app, "remote-ssh-04-delete-dialog")
    confirmDeleteButton.tap()

    let emptyState = app.staticTexts["No SSH profiles saved yet."]
    waitForExists(emptyState, "Expected empty state after delete")
    capture(app, "remote-ssh-05-empty-after-delete")
  }
}
