use del_gl::gl as gl;
use rand::Rng;

fn main() {
    let (tri2vtx, vtx2xyz) = del_msh::io_obj::load_tri_mesh(
        "asset/bunny_1k.obj", Some(1.5));

    let samples = {
        let tri2adjtri = del_msh::topology_uniform::elsuel2(
            &tri2vtx, 3,
            &[0,2,4,6], &[1,2,2,0,0,1],
            vtx2xyz.len() / 3);
        let cumulative_area_sum = del_msh::sampling::cumulative_area_sum(&vtx2xyz, &tri2vtx);
        let mut tri2smpl = std::collections::HashMap::<usize, Vec<usize>>::new();
        let rad = 0.15_f32;
        let mut rng = rand::thread_rng();
        let mut samples = Vec::<(usize, f32, f32)>::new();
        for _itr in 0..1000 {
            let sample_i = del_msh::sampling::sample_uniform(
                &cumulative_area_sum, rng.gen::<f32>(), rng.gen::<f32>());
            let is_near = del_misc::srch_bruteforce::is_there_point_on_mesh_inside_sphere(
                &sample_i, rad, &samples, &tri2smpl,
                &vtx2xyz, &tri2vtx, &tri2adjtri);
            if !is_near {
                match tri2smpl.get_mut(&sample_i.0) {
                    Some(v) => { v.push(samples.len()); },
                    None => { tri2smpl.insert(sample_i.0, vec!(samples.len())); }
                }
                samples.push(sample_i);
            }
        }
        samples
    };
    println!("{}", samples.len());

    let (mut viewer, event_loop) = del_gl::glutin::viewer3::Viewer3::open();
    let mut drawer_mesh = del_gl::mesh::Drawer::new();
    {
        drawer_mesh.compile_shader(&viewer.gl);
        drawer_mesh.update_vertex(&viewer.gl, &vtx2xyz, 3);
        drawer_mesh.add_element(&viewer.gl, gl::TRIANGLES, &tri2vtx, [1., 1., 1.]);
        {
            let line2vtx: Vec<usize> = del_msh::topology_uniform::mshline(
                &tri2vtx, 3,
                &[0, 1, 1, 2, 2, 0],
                vtx2xyz.len() / 3);
            drawer_mesh.add_element(&viewer.gl, gl::LINES, &line2vtx, [0., 0., 0.]);
        }
    }

    let mut drawer_sphere = del_gl::mesh::Drawer::new();
    {
        let sphere_meshtri3 = del_msh::primitive::sphere_tri3::<f32>(
            1., 8, 16);
        drawer_sphere.compile_shader(&viewer.gl);
        drawer_sphere.update_vertex(&viewer.gl, &(sphere_meshtri3.0), 3);
        drawer_sphere.add_element(&viewer.gl, gl::TRIANGLES, &sphere_meshtri3.1, [1., 0., 0.]);
    }
    let mut transform_sphere = del_misc::nalgebra::scale_rot_trans::ScaleRotTrans::new();
    transform_sphere.s = 0.03;

    // this clousure captures drawer, viewer and 'move' them. drawer and viewer cannot be usable anymore
    let event_handle_closure = move |event: glutin::event::Event<()>,
                                     _elwt: &glutin::event_loop::EventLoopWindowTarget<()>,
                                     control_flow: &mut glutin::event_loop::ControlFlow| {
        viewer.event_handle(&event);
        use glutin::event_loop::ControlFlow::{Wait, ExitWithCode};

        if viewer.should_close { *control_flow = ExitWithCode(0); } else { *control_flow = Wait; }
        if viewer.should_draw {
            let mat_projection = viewer.nav.projection_matrix(
                viewer.ui_state.win_width, viewer.ui_state.win_height);
            let mat_modelview = viewer.nav.modelview_matrix();
            drawer_mesh.draw(
                &viewer.gl,
                mat_modelview.as_slice(), // nalgebra is column major same as OpenGL
                mat_projection.as_slice()); // nalgebra is column major same as OpenGL
            for pom in samples.iter() {
                let pos = del_msh::sampling::position_on_mesh_tri3(
                    pom.0, pom.1, pom.2,
                    &vtx2xyz, &tri2vtx);
                transform_sphere.translation = nalgebra::geometry::Translation3::new(
                    pos[0], pos[1], pos[2]);
                let mat_mvo = mat_modelview * transform_sphere.to_homogenous();
                drawer_sphere.draw(
                    &viewer.gl,
                    mat_mvo.as_slice(), // nalgebra is column major same as OpenGL
                    mat_projection.as_slice()); // nalgebra is column major same as OpenGL
            }
            viewer.windowed_context.swap_buffers().unwrap();
        }
    };
    event_loop.run(event_handle_closure);
}