//! An example of generating an image with a compute shader, storing it in a buffer, and then writing it to disk
//! This should produce a file called 'test.png' containing an image of the Mandelbrot fractal

use std::time::Instant;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::{Device, DeviceExtensions};
use vulkano::instance::{Instance, InstanceExtensions, PhysicalDevice};
use vulkano::pipeline::ComputePipeline;
use vulkano::sync;
use vulkano::sync::GpuFuture;

use wasm_vk::*;

use std::sync::Arc;

// Image is SIZExSIZE
const SIZE: usize = 1024;
const BUFFER_SIZE: usize = SIZE*SIZE;

fn slurp(path: impl AsRef<std::path::Path>) -> String {
    use std::fs::File;
    use std::io::Read;
    let mut buf = String::new();
    let mut f = File::open(path).unwrap();
    f.read_to_string(&mut buf).unwrap();
    buf
}

fn main() {
    // We load the '.wat' instead of a '.wasm' for this one
    let w = wabt::wat2wasm(slurp("examples/image.wat")).unwrap();
    let w = wasm::deserialize_buffer(&w).unwrap();

    let ctx = spirv::Ctx::new();
    let m = ctx.module(&w);
    let spv = spirv::module_bytes(m);

    // // Read SPIR-V from file instead of generating it - for debugging
    // let spv = {
    //     use std::io::Read;
    //     let mut f = std::fs::File::open("examples/comp.spv").unwrap();
    //     let mut buf = Vec::new();
    //     f.read_to_end(&mut buf).unwrap();
    //     buf
    // };

    // First, we generate SPIR-V
    // let spv = spirv::to_spirv(w.clone());

    // We write the SPIR-V to disk so we can disassemble it later if we want
    use std::io::Write;
    let mut f = std::fs::File::create("examples/image.spv").unwrap();
    f.write_all(&spv).unwrap();

    println!("Written generated spirv to 'examples/image.spv'");

    // Here's the data we'll be using, it's just BUFFER_SIZE consecutive u32s, starting at 0
    let data_iter = 0..BUFFER_SIZE as u32;

    // Now we'll run the SPIR-V on the GPU with Vulkano.
    // This is a bunch of boilerplate, see the Vulkano examples for explanations.

    let instance = Instance::new(None, &InstanceExtensions::none(), None).unwrap();
    let physical = PhysicalDevice::enumerate(&instance).next().unwrap();
    let queue_family = physical
        .queue_families()
        .find(|&q| q.supports_compute())
        .unwrap();

    let (device, mut queues) = Device::new(
        physical,
        physical.supported_features(),
        &DeviceExtensions::none(),
        [(queue_family, 0.5)].iter().cloned(),
    )
    .unwrap();

    let queue = queues.next().unwrap();

    // This is pretty messy, but is pretty much what you need to do to get your own SPIR-V loaded with Vulkano
    let pipeline = Arc::new({
        #[derive(Copy, Clone)]
        struct PLayout;
        unsafe impl vulkano::descriptor::pipeline_layout::PipelineLayoutDesc for PLayout {
            fn num_sets(&self) -> usize {
                1
            }
            fn num_bindings_in_set(&self, set: usize) -> Option<usize> {
                assert_eq!(set, 0);
                Some(1)
            }
            fn descriptor(
                &self,
                set: usize,
                _binding: usize,
            ) -> Option<vulkano::descriptor::descriptor::DescriptorDesc> {
                assert_eq!(self.num_bindings_in_set(set), Some(1));
                Some(vulkano::descriptor::descriptor::DescriptorDesc {
                    ty: vulkano::descriptor::descriptor::DescriptorDescTy::Buffer(
                        vulkano::descriptor::descriptor::DescriptorBufferDesc {
                            // I have no idea what these do
                            dynamic: Some(false),
                            storage: true,
                        },
                    ),
                    array_count: 1,
                    stages: vulkano::descriptor::descriptor::ShaderStages::compute(),
                    readonly: false,
                })
            }
            fn num_push_constants_ranges(&self) -> usize {
                0
            }
            fn push_constants_range(
                &self,
                _num: usize,
            ) -> Option<vulkano::descriptor::pipeline_layout::PipelineLayoutDescPcRange>
            {
                None
            }
        }

        let shader =
            unsafe { vulkano::pipeline::shader::ShaderModule::new(device.clone(), &spv).unwrap() };

        let entry_str = std::ffi::CString::new("main").unwrap();

        let entry = unsafe { shader.compute_entry_point(&entry_str, PLayout) };

        ComputePipeline::new(device.clone(), &entry, &()).unwrap()
    });

    let data_buffer =
        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), data_iter.clone())
            .unwrap();

    let set = Arc::new(
        PersistentDescriptorSet::start(pipeline.clone(), 0)
            .add_buffer(data_buffer.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    let command_buffer =
        AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family())
            .unwrap()
            // Our workgroups are 64x1x1
            .dispatch([BUFFER_SIZE as u32 / 64, 1, 1], pipeline.clone(), set.clone(), ())
            .unwrap()
            // Finish building the command buffer by calling `build`.
            .build()
            .unwrap();

    // We time it from command buffer submission to fence signaling
    let time = std::time::Instant::now();

    let future = sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    future.wait(None).unwrap();

    let gpu_time = Instant::now() - time;

    // Here's the data the GPU got
    let data_buffer_content = data_buffer.read().unwrap();

    // Print the results (but only show the first 12 values of each):
    println!(
        "Ran in {:?}",
        gpu_time,
    );

    let image = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
        SIZE as u32,
        SIZE as u32,
        data_buffer_content
            .iter()
            .flat_map(|x| x.to_le_bytes().to_vec())
            .collect::<Vec<_>>(),
    )
    .unwrap();
    image.save("test.png").unwrap();
}
