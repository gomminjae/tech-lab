//
//  CanvasView.swift
//  DrawExample
//

import UIKit

/// Manages an offscreen CGContext bitmap and blits it to the screen.
/// Supports both full and incremental rendering modes.
final class CanvasView: UIView {

    private var offscreenContext: CGContext?

    /// Current viewport state (tracked for incremental rendering).
    private(set) var currentScale: Double = 1.0
    private(set) var currentOffsetX: Double = 0.0
    private(set) var currentOffsetY: Double = 0.0

    private var screenScale: CGFloat {
        window?.screen.scale ?? UIScreen.main.scale
    }

    override init(frame: CGRect) {
        super.init(frame: frame)
        backgroundColor = .white
        isOpaque = true
        contentMode = .redraw
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func layoutSubviews() {
        super.layoutSubviews()
        recreateContextIfNeeded()
    }

    // MARK: - Rendering

    /// Perform a full render: process all commands (Clear + SaveState + SetTransform + Draws + RestoreState).
    func applyFullRender(commands: [FfiRenderCommand]) {
        recreateContextIfNeeded()
        guard let ctx = offscreenContext else { return }

        // Extract viewport state from SetTransform command
        for cmd in commands {
            if case let .setTransform(scale, translateX, translateY) = cmd {
                currentScale = scale
                currentOffsetX = translateX
                currentOffsetY = translateY
                break
            }
        }

        RenderCommandProcessor.process(
            commands: commands,
            in: ctx,
            screenScale: screenScale
        )
        setNeedsDisplay()
    }

    /// Perform an incremental render: draw only new segments with the current viewport transform.
    func applyIncrementalRender(commands: [FfiRenderCommand]) {
        guard let ctx = offscreenContext, !commands.isEmpty else { return }

        RenderCommandProcessor.processIncremental(
            commands: commands,
            in: ctx,
            screenScale: screenScale,
            engineScale: currentScale,
            offsetX: currentOffsetX,
            offsetY: currentOffsetY
        )
        setNeedsDisplay()
    }

    // MARK: - Draw

    override func draw(_ rect: CGRect) {
        guard let ctx = offscreenContext,
              let image = ctx.makeImage(),
              let drawCtx = UIGraphicsGetCurrentContext()
        else { return }

        drawCtx.saveGState()

        // CGContext.draw draws with bottom-left origin; flip to match UIKit
        let destRect = CGRect(origin: .zero, size: bounds.size)
        drawCtx.translateBy(x: 0, y: bounds.height)
        drawCtx.scaleBy(x: 1, y: -1)
        drawCtx.draw(image, in: destRect)

        drawCtx.restoreGState()
    }

    // MARK: - Context Management

    private func recreateContextIfNeeded() {
        let scale = screenScale
        let pixelWidth = Int(bounds.width * scale)
        let pixelHeight = Int(bounds.height * scale)

        guard pixelWidth > 0, pixelHeight > 0 else { return }

        if let ctx = offscreenContext,
           ctx.width == pixelWidth,
           ctx.height == pixelHeight {
            return
        }

        let colorSpace = CGColorSpaceCreateDeviceRGB()
        offscreenContext = CGContext(
            data: nil,
            width: pixelWidth,
            height: pixelHeight,
            bitsPerComponent: 8,
            bytesPerRow: pixelWidth * 4,
            space: colorSpace,
            bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
        )

        if let ctx = offscreenContext {
            ctx.setFillColor(CGColor(red: 1, green: 1, blue: 1, alpha: 1))
            ctx.fill(CGRect(x: 0, y: 0, width: pixelWidth, height: pixelHeight))
        }
    }
}
