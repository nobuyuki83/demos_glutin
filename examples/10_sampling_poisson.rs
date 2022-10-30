use del_gl::gl as gl;
use rand::Rng;

fn main() {
    let tri_vtx: Vec<usize>;
    let vtx_xyz: Vec<f32> = {
        let filename: &str = "asset/bunny_1k.obj";
        let mut obj = del_msh::io_obj::WavefrontObj::<f32>::new();
        obj.load(filename);
        println!("vertex size: {}", obj.vtx_xyz.len() / 3);
        println!("element size: {}", obj.elem_vtx_index.len() - 1);
        tri_vtx = obj.elem_vtx_xyz;
        obj.vtx_xyz.iter().map(|v| *v * 0.03).collect()
    };

    let points = {
        let tri_adjtri = del_msh::topology_uniform::elsuel2(
            &tri_vtx, 3,
            &[0,2,4,6], &[1,2,2,0,0,1],
            vtx_xyz.len() / 3);
        let cumulative_area_sum = del_msh::sampling::cumulative_area_sum(&vtx_xyz, &tri_vtx);
        let mut el_smpl = std::collections::HashMap::<usize, Vec<usize>>::new();
        let rad = 0.1_f32;
        let mut rng = rand::thread_rng();
        let mut points = Vec::<(usize, f32, f32)>::new();
        for _itr in 0..1000 {
            let smpli = del_msh::sampling::sample_uniform(
                &cumulative_area_sum, rng.gen::<f32>(), rng.gen::<f32>());
            let is_near = del_misc::srch_bruteforce::is_there_point_on_mesh_inside_sphere(
                &smpli, rad, &points, &el_smpl,
                &vtx_xyz, &tri_vtx, &tri_adjtri);
            if !is_near {
                match el_smpl.get_mut(&smpli.0) {
                    Some(v) => { v.push(points.len()); },
                    None => { el_smpl.insert(smpli.0, vec!(points.len())); }
                }
                points.push(smpli);
            }
        }
        points
    };
    println!("{}", points.len());

    let (mut viewer, event_loop) = del_gl::glutin::viewer3::Viewer3::open();
    let mut drawer_mesh = del_gl::drawer_meshpos::DrawerMeshPos::new();
    {
        drawer_mesh.compile_shader(&viewer.gl);
        drawer_mesh.update_vertex(&viewer.gl, &vtx_xyz, 3);
        drawer_mesh.add_element(&viewer.gl, gl::TRIANGLES, &tri_vtx, [1., 1., 1.]);
        {
            let line_vtx: Vec<usize> = del_msh::topology_uniform::mshline(
                &tri_vtx, 3,
                &[0, 1, 1, 2, 2, 0],
                vtx_xyz.len() / 3);
            drawer_mesh.add_element(&viewer.gl, gl::LINES, &line_vtx, [0., 0., 0.]);
        }
    }

    let mut drawer_sphere = del_gl::drawer_meshpos::DrawerMeshPos::new();
    {
        let sphere_meshtri3 = del_msh::primitive::sphere_tri3::<f32>(
            1., 8, 16);
        drawer_sphere.compile_shader(&viewer.gl);
        drawer_sphere.update_vertex(&viewer.gl, &(sphere_meshtri3.0), 3);
        drawer_sphere.add_element(&viewer.gl, gl::TRIANGLES, &sphere_meshtri3.1, [1., 0., 0.]);
    }
    let mut transform_sphere = del_misc::nalgebra::scale_rot_trans::ScaleRotTrans::new();
    transform_sphere.s = 0.01;

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
            for pom in points.iter() {
                let pos = del_msh::sampling::position_on_mesh_tri3(
                    pom.0, pom.1, pom.2,
                    &vtx_xyz, &tri_vtx);
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