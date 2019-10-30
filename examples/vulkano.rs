//! This example is mostly copied from the Vulkano compute shader example
//! But I removed the comments because this isn't for learning Vulkano, it's for demonstrating wasm-vk
//! If you'd like to learn about the Vulkano part, see the Vulkano examples

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

const BUFFER_SIZE: usize = 65536;

fn main() {
    // We get our WASM from the 'comp.wasm' file, which is compiled from 'comp.wat'
    // It multiplies every number by 12 and adds 3
    let w = wasm::deserialize_file("examples/comp.wasm").unwrap();

    // First, we generate SPIR-V
    let spv = spirv::to_spirv(w.clone());

    // We write the SPIR-V to disk so we can disassemble it later if we want
    use std::io::Write;
    let mut f = std::fs::File::create("examples/comp.spv").unwrap();
    f.write_all(&spv).unwrap();

    println!("Written generated spirv to 'examples/comp.spv'");

    // Here's the data we'll be using, it's just BUFFER_SIZE consecutive u32s, starting at 0
    let data_iter = (0..BUFFER_SIZE as u32);

    // We'll interpret the WASM on the CPU, and time it
    let time = Instant::now();
    // We just pass `interpret` the buffer, and the `wasm::Module`, and it gives us back the new buffer
    let cpu_content = interpret(&data_iter.clone().collect::<Vec<_>>(), &w);
    let cpu_time = Instant::now() - time;

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
            .dispatch([1024, 1, 1], pipeline.clone(), set.clone(), ())
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
        "GPU compiled in {:?}:\n\t{:?}",
        gpu_time,
        &data_buffer_content[..12]
    );
    println!(
        "CPU interpreted in {:?}:\n\t{:?}",
        cpu_time,
        &cpu_content[..12]
    );
}
