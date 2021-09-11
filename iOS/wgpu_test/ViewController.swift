//
//  ViewController.swift
//  wgpu_test
//
//  Created by LiJinlei on 2021/9/10.
//

import UIKit
//Library目录
let library_directory = NSHomeDirectory() + "/Library/"
// 临时目录
let temporary_directory = NSTemporaryDirectory()

class ViewController: UIViewController {
    @IBOutlet var metalV: MetalView!
    var wgpuCanvas: OpaquePointer?
    lazy var displayLink: CADisplayLink = {
        let link = CADisplayLink.init(target: self, selector: #selector(enterFrame))
        return link
    }()
    
    override func viewDidLoad() {
        super.viewDidLoad()
       
        let viewPointer = UnsafeMutableRawPointer(Unmanaged.passRetained(self.metalV).toOpaque())
        let metalLayer = UnsafeMutableRawPointer(Unmanaged.passRetained(self.metalV.layer).toOpaque())
        let maximumFrames = UIScreen.main.maximumFramesPerSecond
        
        let viewObj = ios_obj(view: viewPointer, metal_layer: metalLayer,maximum_frames: Int32(maximumFrames), temporary_directory: temporary_directory, callback_to_swift: callback_to_swift)
        
        wgpuCanvas = create_wgpu_canvas(viewObj)
        self.displayLink.add(to: .current, forMode: .default)
        self.displayLink.isPaused = true
    }
    
    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)
        self.displayLink.isPaused = false
    }
    
    override func viewWillDisappear(_ animated: Bool) {
        super.viewWillDisappear(animated)
        displayLink.isPaused = true
    }
    
    @objc func enterFrame() {
        guard let canvas = self.wgpuCanvas else {
            return
        }
        // call rust
        enter_frame(canvas)
    }

}

func callback_to_swift(arg: Int32) {
    DispatchQueue.main.async {
        switch arg {
        case 0:
            print("wgpu canvas created!")
            break
        case 1:
            break
            
        default:
            break
        }
    }
    
}
