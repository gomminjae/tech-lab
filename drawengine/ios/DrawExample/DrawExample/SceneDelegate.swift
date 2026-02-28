//
//  SceneDelegate.swift
//  DrawExample
//
//  Created by 권민재 on 3/1/26.
//

import UIKit

class SceneDelegate: UIResponder, UIWindowSceneDelegate {

    var window: UIWindow?

    func scene(_ scene: UIScene, willConnectTo session: UISceneSession, options connectionOptions: UIScene.ConnectionOptions) {
        guard let windowScene = scene as? UIWindowScene else { return }

        let window = UIWindow(windowScene: windowScene)
        window.rootViewController = CanvasViewController()
        window.makeKeyAndVisible()
        self.window = window
    }
}
