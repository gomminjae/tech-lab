//
//  RenderCommandProcessor.swift
//  DrawExample
//

import CoreGraphics

/// Converts FfiRenderCommand sequences into CoreGraphics drawing calls.
/// Handles variable-width bezier tessellation by building closed outline polygons.
final class RenderCommandProcessor {

    private static let samplesPerSegment = 8

    /// Process a full render command sequence (Clear + SaveState + SetTransform + Draws + RestoreState).
    static func process(
        commands: [FfiRenderCommand],
        in context: CGContext,
        screenScale: CGFloat
    ) {
        for command in commands {
            switch command {
            case let .clear(r, g, b, a):
                context.saveGState()
                context.concatenate(context.ctm.inverted())
                let rect = CGRect(
                    x: 0, y: 0,
                    width: context.width,
                    height: context.height
                )
                context.setBlendMode(.normal)
                context.setFillColor(cgColor(r: r, g: g, b: b, a: a))
                context.fill(rect)
                context.restoreGState()

            case .saveState:
                context.saveGState()

            case .restoreState:
                context.restoreGState()

            case let .setTransform(scale, translateX, translateY):
                applyTransform(
                    context: context,
                    engineScale: scale,
                    translateX: translateX,
                    translateY: translateY,
                    screenScale: screenScale
                )

            case let .drawVariableWidthPath(segments, r, g, b, a, isEraser):
                drawPath(
                    context: context,
                    segments: segments,
                    r: r, g: g, b: b, a: a,
                    isEraser: isEraser
                )
            }
        }
    }

    /// Process incremental render commands (only DrawVariableWidthPath).
    /// Wraps them with the current viewport transform.
    static func processIncremental(
        commands: [FfiRenderCommand],
        in context: CGContext,
        screenScale: CGFloat,
        engineScale: Double,
        offsetX: Double,
        offsetY: Double
    ) {
        guard !commands.isEmpty else { return }

        context.saveGState()
        applyTransform(
            context: context,
            engineScale: engineScale,
            translateX: offsetX,
            translateY: offsetY,
            screenScale: screenScale
        )

        for command in commands {
            if case let .drawVariableWidthPath(segments, r, g, b, a, isEraser) = command {
                drawPath(
                    context: context,
                    segments: segments,
                    r: r, g: g, b: b, a: a,
                    isEraser: isEraser
                )
            }
        }

        context.restoreGState()
    }

    // MARK: - Transform

    private static func applyTransform(
        context: CGContext,
        engineScale: Double,
        translateX: Double,
        translateY: Double,
        screenScale: CGFloat
    ) {
        context.concatenate(context.ctm.inverted())

        // CGContext origin is bottom-left; UIKit/engine origin is top-left.
        // Flip Y: translate to bottom edge, then negate Y scale.
        context.translateBy(x: 0, y: CGFloat(context.height))
        context.scaleBy(x: screenScale, y: -screenScale)

        // Apply engine viewport: pan then zoom
        context.translateBy(x: CGFloat(translateX), y: CGFloat(translateY))
        context.scaleBy(x: CGFloat(engineScale), y: CGFloat(engineScale))
    }

    // MARK: - Path Drawing

    private static func drawPath(
        context: CGContext,
        segments: [FfiPathSegment],
        r: Float, g: Float, b: Float, a: Float,
        isEraser: Bool
    ) {
        guard !segments.isEmpty else { return }

        context.setBlendMode(isEraser ? .clear : .normal)
        context.setFillColor(cgColor(r: r, g: g, b: b, a: a))

        // Single degenerate segment → draw a dot
        if segments.count == 1 {
            let seg = segments[0]
            let dx = seg.p3X - seg.p0X
            let dy = seg.p3Y - seg.p0Y
            if dx * dx + dy * dy < 0.01 {
                let radius = seg.startWidth / 2.0
                context.fillEllipse(in: CGRect(
                    x: seg.p0X - radius, y: seg.p0Y - radius,
                    width: radius * 2, height: radius * 2
                ))
                return
            }
        }

        for segment in segments {
            tessellateAndFill(context: context, segment: segment)
        }
    }

    // MARK: - Tessellation

    /// Tessellates a cubic bezier segment into a filled polygon representing
    /// the variable-width stroke outline.
    private static func tessellateAndFill(
        context: CGContext,
        segment: FfiPathSegment
    ) {
        let n = samplesPerSegment
        var leftPoints = [CGPoint]()
        var rightPoints = [CGPoint]()
        leftPoints.reserveCapacity(n + 1)
        rightPoints.reserveCapacity(n + 1)

        let p0x = segment.p0X, p0y = segment.p0Y
        let c1x = segment.cp1X, c1y = segment.cp1Y
        let c2x = segment.cp2X, c2y = segment.cp2Y
        let p3x = segment.p3X, p3y = segment.p3Y

        for i in 0...n {
            let t = Double(i) / Double(n)
            let mt = 1.0 - t
            let mt2 = mt * mt
            let mt3 = mt2 * mt
            let t2 = t * t
            let t3 = t2 * t

            // Cubic bezier position: B(t)
            let x = mt3 * p0x + 3.0 * mt2 * t * c1x + 3.0 * mt * t2 * c2x + t3 * p3x
            let y = mt3 * p0y + 3.0 * mt2 * t * c1y + 3.0 * mt * t2 * c2y + t3 * p3y

            // Derivative (tangent): B'(t)
            let dx = 3.0 * mt2 * (c1x - p0x) + 6.0 * mt * t * (c2x - c1x) + 3.0 * t2 * (p3x - c2x)
            let dy = 3.0 * mt2 * (c1y - p0y) + 6.0 * mt * t * (c2y - c1y) + 3.0 * t2 * (p3y - c2y)

            // Normal (perpendicular to tangent)
            let len = sqrt(dx * dx + dy * dy)
            let nx: Double, ny: Double
            if len > 1e-10 {
                nx = -dy / len
                ny = dx / len
            } else {
                nx = 0.0
                ny = 1.0
            }

            // Linearly interpolated width
            let halfWidth = (segment.startWidth + (segment.endWidth - segment.startWidth) * t) / 2.0

            leftPoints.append(CGPoint(x: x + nx * halfWidth, y: y + ny * halfWidth))
            rightPoints.append(CGPoint(x: x - nx * halfWidth, y: y - ny * halfWidth))
        }

        // Closed polygon: left forward → right backward
        context.beginPath()
        context.move(to: leftPoints[0])
        for i in 1..<leftPoints.count {
            context.addLine(to: leftPoints[i])
        }
        for i in stride(from: rightPoints.count - 1, through: 0, by: -1) {
            context.addLine(to: rightPoints[i])
        }
        context.closePath()
        context.fillPath()

        // Round caps at endpoints
        let sr = segment.startWidth / 2.0
        let er = segment.endWidth / 2.0
        context.fillEllipse(in: CGRect(x: p0x - sr, y: p0y - sr, width: sr * 2, height: sr * 2))
        context.fillEllipse(in: CGRect(x: p3x - er, y: p3y - er, width: er * 2, height: er * 2))
    }

    // MARK: - Helpers

    private static func cgColor(r: Float, g: Float, b: Float, a: Float) -> CGColor {
        CGColor(red: CGFloat(r), green: CGFloat(g), blue: CGFloat(b), alpha: CGFloat(a))
    }
}
