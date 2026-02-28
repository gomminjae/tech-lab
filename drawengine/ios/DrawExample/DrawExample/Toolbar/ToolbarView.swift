//
//  ToolbarView.swift
//  DrawExample
//

import UIKit

protocol ToolbarViewDelegate: AnyObject {
    func toolbarDidSelectBrush(_ type: FfiBrushType, color: FfiColor, width: Double)
    func toolbarDidTapUndo()
    func toolbarDidTapRedo()
    func toolbarDidTapSave()
    func toolbarDidTapLoad()
}

/// Horizontal toolbar with brush type, color, width, undo/redo, and save/load controls.
final class ToolbarView: UIView {

    weak var delegate: ToolbarViewDelegate?

    private var selectedBrushType: FfiBrushType = .pen
    private var selectedColor: UIColor = .black
    private var selectedWidth: Double = 3.0

    private let brushSegment = UISegmentedControl(items: ["Pen", "Highlighter", "Eraser"])
    private let colorButton = UIButton(type: .system)
    private let widthSlider = UISlider()
    private let undoButton = UIButton(type: .system)
    private let redoButton = UIButton(type: .system)
    private let saveButton = UIButton(type: .system)
    private let loadButton = UIButton(type: .system)

    override init(frame: CGRect) {
        super.init(frame: frame)
        setupUI()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func updateUndoRedoState(canUndo: Bool, canRedo: Bool) {
        undoButton.isEnabled = canUndo
        redoButton.isEnabled = canRedo
    }

    // MARK: - Setup

    private func setupUI() {
        backgroundColor = UIColor.systemBackground.withAlphaComponent(0.95)
        layer.cornerRadius = 12
        layer.shadowColor = UIColor.black.cgColor
        layer.shadowOpacity = 0.15
        layer.shadowRadius = 8
        layer.shadowOffset = CGSize(width: 0, height: -2)

        brushSegment.selectedSegmentIndex = 0
        brushSegment.addTarget(self, action: #selector(brushChanged), for: .valueChanged)

        colorButton.setImage(
            UIImage(systemName: "circle.fill")?.withRenderingMode(.alwaysTemplate),
            for: .normal
        )
        colorButton.tintColor = selectedColor
        colorButton.addTarget(self, action: #selector(colorTapped), for: .touchUpInside)

        widthSlider.minimumValue = 1.0
        widthSlider.maximumValue = 20.0
        widthSlider.value = Float(selectedWidth)
        widthSlider.addTarget(self, action: #selector(widthChanged), for: .valueChanged)
        widthSlider.widthAnchor.constraint(equalToConstant: 80).isActive = true

        undoButton.setImage(UIImage(systemName: "arrow.uturn.backward"), for: .normal)
        undoButton.addTarget(self, action: #selector(undoTapped), for: .touchUpInside)
        undoButton.isEnabled = false

        redoButton.setImage(UIImage(systemName: "arrow.uturn.forward"), for: .normal)
        redoButton.addTarget(self, action: #selector(redoTapped), for: .touchUpInside)
        redoButton.isEnabled = false

        saveButton.setImage(UIImage(systemName: "square.and.arrow.down"), for: .normal)
        saveButton.addTarget(self, action: #selector(saveTapped), for: .touchUpInside)

        loadButton.setImage(UIImage(systemName: "folder"), for: .normal)
        loadButton.addTarget(self, action: #selector(loadTapped), for: .touchUpInside)

        let stack = UIStackView(arrangedSubviews: [
            brushSegment, colorButton, widthSlider,
            makeSeparator(),
            undoButton, redoButton,
            makeSeparator(),
            saveButton, loadButton,
        ])
        stack.axis = .horizontal
        stack.alignment = .center
        stack.spacing = 8
        stack.translatesAutoresizingMaskIntoConstraints = false

        addSubview(stack)
        NSLayoutConstraint.activate([
            stack.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 12),
            stack.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -12),
            stack.centerYAnchor.constraint(equalTo: centerYAnchor),
        ])
    }

    private func makeSeparator() -> UIView {
        let sep = UIView()
        sep.backgroundColor = .separator
        sep.translatesAutoresizingMaskIntoConstraints = false
        sep.widthAnchor.constraint(equalToConstant: 1).isActive = true
        sep.heightAnchor.constraint(equalToConstant: 28).isActive = true
        return sep
    }

    // MARK: - Actions

    @objc private func brushChanged() {
        switch brushSegment.selectedSegmentIndex {
        case 0: selectedBrushType = .pen
        case 1: selectedBrushType = .highlighter
        case 2: selectedBrushType = .eraser
        default: break
        }
        notifyBrushChange()
    }

    @objc private func colorTapped() {
        guard let vc = findViewController() else { return }
        let picker = UIColorPickerViewController()
        picker.selectedColor = selectedColor
        picker.delegate = self
        vc.present(picker, animated: true)
    }

    @objc private func widthChanged() {
        selectedWidth = Double(widthSlider.value)
        notifyBrushChange()
    }

    @objc private func undoTapped() { delegate?.toolbarDidTapUndo() }
    @objc private func redoTapped() { delegate?.toolbarDidTapRedo() }
    @objc private func saveTapped() { delegate?.toolbarDidTapSave() }
    @objc private func loadTapped() { delegate?.toolbarDidTapLoad() }

    private func notifyBrushChange() {
        var r: CGFloat = 0, g: CGFloat = 0, b: CGFloat = 0, a: CGFloat = 0
        selectedColor.getRed(&r, green: &g, blue: &b, alpha: &a)
        let color = FfiColor(r: Float(r), g: Float(g), b: Float(b), a: Float(a))
        delegate?.toolbarDidSelectBrush(selectedBrushType, color: color, width: selectedWidth)
    }

    private func findViewController() -> UIViewController? {
        var responder: UIResponder? = self
        while let next = responder?.next {
            if let vc = next as? UIViewController { return vc }
            responder = next
        }
        return nil
    }
}

// MARK: - UIColorPickerViewControllerDelegate

extension ToolbarView: UIColorPickerViewControllerDelegate {
    func colorPickerViewController(
        _ viewController: UIColorPickerViewController,
        didSelect color: UIColor,
        continuously: Bool
    ) {
        selectedColor = color
        colorButton.tintColor = color
        notifyBrushChange()
    }
}
