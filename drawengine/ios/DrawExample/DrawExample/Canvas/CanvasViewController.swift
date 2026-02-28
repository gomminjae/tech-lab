//
//  CanvasViewController.swift
//  DrawExample
//

import UIKit

/// Main view controller: manages the canvas, toolbar, and engine.
/// Routes touch input to DrawEngineFFI and renders the results.
final class CanvasViewController: UIViewController {

    private let canvasView = CanvasView()
    private let toolbarView = ToolbarView()
    private var engine: DrawEngineFfi!
    private var isStrokeActive = false

    // MARK: - Lifecycle

    override func viewDidLoad() {
        super.viewDidLoad()
        view.backgroundColor = .white
        setupCanvas()
        setupToolbar()
        setupGestures()
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()
        if engine == nil {
            initializeEngine()
        }
    }

    // MARK: - Setup

    private func setupCanvas() {
        canvasView.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(canvasView)
        NSLayoutConstraint.activate([
            canvasView.topAnchor.constraint(equalTo: view.topAnchor),
            canvasView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            canvasView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            canvasView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])
    }

    private func setupToolbar() {
        toolbarView.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(toolbarView)
        NSLayoutConstraint.activate([
            toolbarView.leadingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.leadingAnchor, constant: 8),
            toolbarView.trailingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.trailingAnchor, constant: -8),
            toolbarView.bottomAnchor.constraint(equalTo: view.safeAreaLayoutGuide.bottomAnchor, constant: -8),
            toolbarView.heightAnchor.constraint(equalToConstant: 52),
        ])
        toolbarView.delegate = self
    }

    private func setupGestures() {
        let pinch = UIPinchGestureRecognizer(target: self, action: #selector(handlePinch(_:)))
        pinch.delegate = self
        canvasView.addGestureRecognizer(pinch)

        let pan = UIPanGestureRecognizer(target: self, action: #selector(handlePan(_:)))
        pan.minimumNumberOfTouches = 2
        pan.delegate = self
        canvasView.addGestureRecognizer(pan)
    }

    private func initializeEngine() {
        let size = canvasView.bounds.size
        guard size.width > 0, size.height > 0 else { return }
        engine = DrawEngineFfi(width: Double(size.width), height: Double(size.height))
        let commands = engine.fullRender()
        canvasView.applyFullRender(commands: commands)
        updateToolbarState()
    }

    // MARK: - Touch Handling (1-finger drawing)

    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard let touch = touches.first, touches.count == 1, engine != nil else { return }

        if touch.type == .pencil || event?.allTouches?.count == 1 {
            let location = touch.location(in: canvasView)
            let pressure = pressureValue(for: touch)

            let commands = engine.beginStroke(
                x: Double(location.x),
                y: Double(location.y),
                pressure: pressure,
                timestamp: touch.timestamp
            )
            isStrokeActive = true
            canvasView.applyIncrementalRender(commands: commands)
        }
    }

    override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard let touch = touches.first, isStrokeActive, engine != nil else { return }

        let coalescedTouches = event?.coalescedTouches(for: touch) ?? [touch]
        for coalesced in coalescedTouches {
            let location = coalesced.location(in: canvasView)
            let pressure = pressureValue(for: coalesced)

            let commands = engine.addPoint(
                x: Double(location.x),
                y: Double(location.y),
                pressure: pressure,
                timestamp: coalesced.timestamp
            )
            canvasView.applyIncrementalRender(commands: commands)
        }
    }

    override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard isStrokeActive, engine != nil else { return }
        finishStroke()
    }

    override func touchesCancelled(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard isStrokeActive, engine != nil else { return }
        finishStroke()
    }

    private func finishStroke() {
        isStrokeActive = false
        let commands = engine.endStroke()
        canvasView.applyFullRender(commands: commands)
        updateToolbarState()
    }

    private func pressureValue(for touch: UITouch) -> Double {
        if touch.maximumPossibleForce > 0 {
            return Double(touch.force / touch.maximumPossibleForce)
        }
        return 0.5
    }

    // MARK: - Gesture Handling (2-finger zoom/pan)

    @objc private func handlePinch(_ gesture: UIPinchGestureRecognizer) {
        guard engine != nil else { return }

        switch gesture.state {
        case .began:
            cancelActiveStrokeIfNeeded()
        case .changed:
            let center = gesture.location(in: canvasView)
            let commands = engine.zoom(
                factor: Double(gesture.scale),
                focalX: Double(center.x),
                focalY: Double(center.y)
            )
            gesture.scale = 1.0
            canvasView.applyFullRender(commands: commands)
        default:
            break
        }
    }

    private var lastPanTranslation: CGPoint = .zero

    @objc private func handlePan(_ gesture: UIPanGestureRecognizer) {
        guard engine != nil else { return }

        switch gesture.state {
        case .began:
            cancelActiveStrokeIfNeeded()
            lastPanTranslation = .zero
        case .changed:
            let translation = gesture.translation(in: canvasView)
            let dx = translation.x - lastPanTranslation.x
            let dy = translation.y - lastPanTranslation.y
            lastPanTranslation = translation

            let commands = engine.pan(dx: Double(dx), dy: Double(dy))
            canvasView.applyFullRender(commands: commands)
        default:
            lastPanTranslation = .zero
        }
    }

    private func cancelActiveStrokeIfNeeded() {
        guard isStrokeActive else { return }
        isStrokeActive = false
        let commands = engine.endStroke()
        canvasView.applyFullRender(commands: commands)
        updateToolbarState()
    }

    // MARK: - State

    private func updateToolbarState() {
        guard engine != nil else { return }
        let state = engine.getState()
        toolbarView.updateUndoRedoState(canUndo: state.canUndo, canRedo: state.canRedo)
    }
}

// MARK: - UIGestureRecognizerDelegate

extension CanvasViewController: UIGestureRecognizerDelegate {
    func gestureRecognizer(
        _ gestureRecognizer: UIGestureRecognizer,
        shouldRecognizeSimultaneouslyWith otherGestureRecognizer: UIGestureRecognizer
    ) -> Bool {
        true
    }
}

// MARK: - ToolbarViewDelegate

extension CanvasViewController: ToolbarViewDelegate {

    func toolbarDidSelectBrush(_ type: FfiBrushType, color: FfiColor, width: Double) {
        guard engine != nil else { return }
        engine.setBrush(config: FfiBrushConfig(brushType: type, color: color, baseWidth: width))
    }

    func toolbarDidTapUndo() {
        guard engine != nil else { return }
        let commands = engine.undo()
        canvasView.applyFullRender(commands: commands)
        updateToolbarState()
    }

    func toolbarDidTapRedo() {
        guard engine != nil else { return }
        let commands = engine.redo()
        canvasView.applyFullRender(commands: commands)
        updateToolbarState()
    }

    func toolbarDidTapSave() {
        guard engine != nil else { return }
        do {
            let json = try engine.save()
            let url = try DocumentManager.save(json: json)
            showAlert(title: "Saved", message: url.lastPathComponent)
        } catch {
            showAlert(title: "Save Failed", message: error.localizedDescription)
        }
    }

    func toolbarDidTapLoad() {
        guard engine != nil else { return }
        do {
            let files = DocumentManager.listFiles()
            guard let latest = files.first else {
                showAlert(title: "No Files", message: "No saved drawings found.")
                return
            }
            let json = try DocumentManager.load(url: latest)
            try engine.load(json: json)
            let commands = engine.fullRender()
            canvasView.applyFullRender(commands: commands)
            updateToolbarState()
            showAlert(title: "Loaded", message: latest.lastPathComponent)
        } catch {
            showAlert(title: "Load Failed", message: error.localizedDescription)
        }
    }

    private func showAlert(title: String, message: String) {
        let alert = UIAlertController(title: title, message: message, preferredStyle: .alert)
        alert.addAction(UIAlertAction(title: "OK", style: .default))
        present(alert, animated: true)
    }
}
