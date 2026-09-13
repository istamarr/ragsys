// use std::sync::Arc;
//
// fn detect_objects_on_image(buf: Vec<u8>) -> Vec<(f32, f32, f32, f32, &'static str, f32)> {
//     println!("log: processing image input preparing");
//     let (input,img_width,img_height) = prepare_input(buf);
//     println!("log: processing image run model");
//     let output = run_model(input);
//     println!("log: processing image ouput");
//     return process_output(output, img_width, img_height);
// }
