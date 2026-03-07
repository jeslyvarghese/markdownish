import Metal
import MetalKit
import CoreGraphics

private let kMetalShaderSource = """
#include <metal_stdlib>
using namespace metal;

struct VertexIn {
    float2 position [[attribute(0)]];
    float2 uv       [[attribute(1)]];
};
struct VertexOut {
    float4 position [[position]];
    float2 uv;
};
struct ScrollUniforms {
    float scrollY;
    float viewportHeight;
    float documentHeight;
    float scale;
};

vertex VertexOut vertex_main(VertexIn in [[stage_in]]) {
    VertexOut out;
    out.position = float4(in.position, 0.0, 1.0);
    out.uv = in.uv;
    return out;
}

fragment float4 fragment_main(
    VertexOut in [[stage_in]],
    texture2d<float> tex [[texture(0)]],
    constant ScrollUniforms& u [[buffer(0)]]
) {
    constexpr sampler s(filter::linear, address::clamp_to_edge);
    float uvOffset = u.scrollY / (u.documentHeight * u.scale);
    return tex.sample(s, float2(in.uv.x, in.uv.y + uvOffset));
}
"""

struct ScrollUniforms {
    var scrollY: Float
    var viewportHeight: Float
    var documentHeight: Float
    var scale: Float
}

final class DocumentRenderer: NSObject, MTKViewDelegate {

    private let device: MTLDevice
    private let commandQueue: MTLCommandQueue
    private let pipelineState: MTLRenderPipelineState
    private let vertexBuffer: MTLBuffer
    private let uniformBuffer: MTLBuffer
    private var texture: MTLTexture?

    private(set) var documentHeight: CGFloat = 0

    var document: MarkdownDocument? { didSet { invalidate() } }
    var renderConfig: RenderConfig { didSet { invalidate() } }
    var scrollY: CGFloat = 0 { didSet { needsDraw = true } }

    private var needsDraw = true

    init?(mtkView: MTKView, config: RenderConfig) {
        guard
            let dev = mtkView.device ?? MTLCreateSystemDefaultDevice(),
            let queue = dev.makeCommandQueue()
        else { return nil }

        self.device = dev
        self.commandQueue = queue
        self.renderConfig = config

        // Compile Metal shader from embedded source
        guard
            let library = try? dev.makeLibrary(source: kMetalShaderSource, options: nil),
            let vertFn   = library.makeFunction(name: "vertex_main"),
            let fragFn   = library.makeFunction(name: "fragment_main")
        else { return nil }

        let vertexDesc = MTLVertexDescriptor()
        vertexDesc.attributes[0].format = .float2; vertexDesc.attributes[0].offset = 0;  vertexDesc.attributes[0].bufferIndex = 0
        vertexDesc.attributes[1].format = .float2; vertexDesc.attributes[1].offset = 8;  vertexDesc.attributes[1].bufferIndex = 0
        vertexDesc.layouts[0].stride = 16

        let pd = MTLRenderPipelineDescriptor()
        pd.vertexFunction   = vertFn
        pd.fragmentFunction = fragFn
        pd.colorAttachments[0].pixelFormat = mtkView.colorPixelFormat
        pd.vertexDescriptor = vertexDesc

        guard let ps = try? dev.makeRenderPipelineState(descriptor: pd) else { return nil }
        self.pipelineState = ps

        // Full-screen quad: NDC position + UV
        let verts: [Float] = [-1, 1, 0, 0,   1, 1, 1, 0,   -1, -1, 0, 1,   1, -1, 1, 1]
        guard
            let vb = dev.makeBuffer(bytes: verts, length: verts.count * 4, options: .storageModeShared),
            let ub = dev.makeBuffer(length: MemoryLayout<ScrollUniforms>.size, options: .storageModeShared)
        else { return nil }
        self.vertexBuffer = vb
        self.uniformBuffer = ub

        super.init()
        mtkView.delegate = self
        mtkView.isPaused = false
        mtkView.enableSetNeedsDisplay = false
    }

    // MARK: - MTKViewDelegate

    func mtkView(_ view: MTKView, drawableSizeWillChange size: CGSize) { invalidate() }

    func draw(in view: MTKView) {
        let dpSize = view.drawableSize
        let scale  = renderConfig.scale

        if needsDraw || texture == nil {
            needsDraw = false
            renderToTexture(w: dpSize.width / scale, h: dpSize.height / scale)
        }

        guard
            let tex      = texture,
            let drawable = view.currentDrawable,
            let rpd      = view.currentRenderPassDescriptor,
            let cmd      = commandQueue.makeCommandBuffer(),
            let enc      = cmd.makeRenderCommandEncoder(descriptor: rpd)
        else { return }

        var uni = ScrollUniforms(
            scrollY:        Float(scrollY),
            viewportHeight: Float(dpSize.height / scale),
            documentHeight: Float(documentHeight),
            scale:          Float(scale)
        )
        memcpy(uniformBuffer.contents(), &uni, MemoryLayout<ScrollUniforms>.size)

        enc.setRenderPipelineState(pipelineState)
        enc.setVertexBuffer(vertexBuffer, offset: 0, index: 0)
        enc.setFragmentTexture(tex, index: 0)
        enc.setFragmentBuffer(uniformBuffer, offset: 0, index: 0)
        enc.drawPrimitives(type: .triangleStrip, vertexStart: 0, vertexCount: 4)
        enc.endEncoding()
        cmd.present(drawable)
        cmd.commit()
    }

    func invalidate() { needsDraw = true; texture = nil }

    // MARK: - Render to texture

    private func renderToTexture(w: CGFloat, h: CGFloat) {
        guard let doc = document else { return }
        documentHeight = LayoutEngine.documentHeight(document: doc, config: renderConfig)
        guard let img = LayoutEngine.render(
            document: doc, scrollY: scrollY,
            viewportWidth: w, viewportHeight: h,
            config: renderConfig
        ) else { return }
        texture = makeTexture(from: img)
    }

    private func makeTexture(from image: CGImage) -> MTLTexture? {
        let w = image.width, h = image.height
        let desc = MTLTextureDescriptor.texture2DDescriptor(
            pixelFormat: .bgra8Unorm, width: w, height: h, mipmapped: false)
        desc.usage = .shaderRead
        guard let tex = device.makeTexture(descriptor: desc) else { return nil }

        let cs   = CGColorSpaceCreateDeviceRGB()
        let info = CGImageAlphaInfo.premultipliedFirst.rawValue | CGBitmapInfo.byteOrder32Little.rawValue
        guard
            let ctx  = CGContext(data: nil, width: w, height: h, bitsPerComponent: 8,
                                 bytesPerRow: w * 4, space: cs, bitmapInfo: info),
            let data = ctx.data
        else { return nil }
        ctx.draw(image, in: CGRect(x: 0, y: 0, width: w, height: h))
        tex.replace(region: MTLRegionMake2D(0, 0, w, h), mipmapLevel: 0, withBytes: data, bytesPerRow: w * 4)
        return tex
    }
}
