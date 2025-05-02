// use ash::vk::{
//     Format, VertexInputAttributeDescription, VertexInputBindingDescription, VertexInputRate,
// };
// #[allow(dead_code)]
// pub struct Vertex {
//     pos: (f32, f32),
//     color: (f32, f32, f32),
// }
// 
// pub fn get_vertices() -> Vec<Vertex> {
//     vec![
//         Vertex {
//             pos: (-0.5, -0.5),
//             color: (1.0, 0.0, 0.0),
//         },
//         Vertex {
//             pos: (0.5, -0.5),
//             color: (0.0, 1.0, 0.0),
//         },
//         Vertex {
//             pos: (0.5, 0.5),
//             color: (1.0, 0.0, 0.0),
//         },
//         Vertex {
//             pos: (-0.5, 0.5),
//             color: (0.0, 0.0, 1.0),
//         },
//     ]
// }
// pub fn get_indices() -> Vec<u16> {
//     vec![0, 1, 2, 2, 3, 0]
// }
// 
// impl Vertex {
//     pub fn get_binding_descriptions() -> [VertexInputBindingDescription; 1] {
//         [VertexInputBindingDescription::default()
//             .binding(0)
//             .stride(size_of::<Vertex>() as u32)
//             .input_rate(VertexInputRate::VERTEX)]
//     }
// 
//     pub fn get_attribute_descriptions() -> [VertexInputAttributeDescription; 2] {
//         [
//             VertexInputAttributeDescription::default()
//                 .binding(0)
//                 .location(0)
//                 .format(Format::R32G32_SFLOAT)
//                 .offset(0),
//             VertexInputAttributeDescription::default()
//                 .binding(0)
//                 .location(1)
//                 .format(Format::R32G32B32_SFLOAT)
//                 .offset(8),
//         ]
//     }
// }
