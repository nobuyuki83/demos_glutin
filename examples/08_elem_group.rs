use del_gl::gl as gl;


/*
            for iel in 0..tri_vtx_uv.len() / 3 {
                println!("{}", iel);
                println!("  {} {} {}", tri_vtx_uv[iel*3+0], tri_vtx_uv[iel*3+1], tri_vtx_uv[iel*3+2]);
                for ifael in 0..3 {
                    let jel = elsuel[iel*3+ifael];
                    println!("   {}", jel);
                    if jel != usize::MAX {
                        println!("      {} {} {}", tri_vtx_uv[jel*3+0], tri_vtx_uv[jel*3+1], tri_vtx_uv[jel*3+2]);
                    }
                }
            }
 */

fn main() {
    let (mut viewer, event_loop) = del_gl::glutin::viewer3::Viewer3::open();

    let mut drawer = del_gl::drawer_meshpos::DrawerMeshPos::new();
    {
        let tri_vtx_xyz: Vec<usize>;
        let vtx_xyz: Vec<f32>;
        let tri_vtx_uv: Vec<usize>;
        let vtx_uv: Vec<f32>;
        {
            let mut obj = del_msh::io_obj::WavefrontObj::<f32>::new();
            let filename: &str = "asset/Babi/babi.obj";
            obj.load(filename);
            let elem_vtx_xyz_index: Vec<usize> = obj.elem_vtx_index.iter().map(|i| *i as usize).collect();
            let num_elem = elem_vtx_xyz_index.len() - 1;
            println!("element size: {}", num_elem);
            assert_eq!(elem_vtx_xyz_index[num_elem-1],(num_elem-1)*3);  // triangle mesh
            tri_vtx_xyz = obj.elem_vtx_xyz.iter().map(|i| *i as usize).collect();
            vtx_xyz = del_misc::nalgebra::msh_misc::centerize_normalize_boundingbox(obj.vtx_xyz, 3);
            vtx_uv = obj.vtx_uv;
            tri_vtx_uv = obj.elem_vtx_uv.iter().map(|i| *i as usize).collect();
        };
        println!("vertex size: {}", vtx_xyz.len() / 3);
        println!("uv size: {}", vtx_uv.len() / 2);
        let (num_group, elem_group) = {
            let elsuel = del_msh::topology_uniform::elsuel2(
                &tri_vtx_uv, 3,
                &[0,2,4,6], &[1,2,2,0,0,1],
                vtx_uv.len() / 2);
            let (num_group, elem_group) = del_msh::group::make_group_elem(&tri_vtx_uv, 3, &elsuel);
            (num_group, elem_group)
        };
        println!("num_group: {}",num_group);
        use crate::gl::types::GLuint;
        drawer.compile_shader(&viewer.gl);
        drawer.update_vertex(&viewer.gl, &vtx_xyz, 3);
        for i_group in 0..num_group {
            let mut tri_vtx0 = Vec::<usize>::new();
            for i_elem in 0..elem_group.len() {
                if elem_group[i_elem] == i_group {
                    tri_vtx0.push(tri_vtx_xyz[i_elem*3+0]);
                    tri_vtx0.push(tri_vtx_xyz[i_elem*3+1]);
                    tri_vtx0.push(tri_vtx_xyz[i_elem*3+2]);
                }
            }
            let tri_vtx1: Vec<GLuint> = tri_vtx0.iter().map(|i| *i as gl::types::GLuint).collect();
            let r = (i_group % 3 + 1) as f32 / 3 as f32;
            let g = (i_group % 4 + 1) as f32 / 4 as f32;
            let b = (i_group % 5 + 1) as f32 / 5 as f32;
            drawer.add_element(&viewer.gl, gl::TRIANGLES, &tri_vtx1, [r, g, b]);
        }
        {
            let line_vtx_xyz: Vec<usize> = del_msh::topology_uniform::mshline(
                &tri_vtx_xyz, 3,
                &[0,1,1,2,2,0],
                vtx_xyz.len() / 3);
            let line_vtx_xyz0: Vec<GLuint> = line_vtx_xyz.iter().map(|i| *i as gl::types::GLuint).collect();
            drawer.add_element(&viewer.gl, gl::LINES, &line_vtx_xyz0, [0., 0., 0.]);
        }
    }

    // this clousure captures drawer, viewer and 'move' them. drawer and viewer cannot be usable anymore
    let event_handle_closure = move |event: glutin::event::Event<()>,
                                     _elwt: &glutin::event_loop::EventLoopWindowTarget<()>,
                                     control_flow: &mut glutin::event_loop::ControlFlow| {
        viewer.event_handle(&event);
        use glutin::event_loop::ControlFlow::{Wait, ExitWithCode};
        if viewer.should_close { *control_flow = ExitWithCode(0); } else { *control_flow = Wait; }
        //
        if viewer.should_draw {
            let mat_projection = viewer.nav.projection_matrix(
                viewer.ui_state.win_width, viewer.ui_state.win_height);
            let mat_modelview = viewer.nav.modelview_matrix();
            drawer.draw(
                &viewer.gl,
                mat_modelview.as_slice(), // nalgebra is column major same as OpenGL
                mat_projection.as_slice()); // nalgebra is column major same as OpenGL
            viewer.windowed_context.swap_buffers().unwrap();
        }
    };
    event_loop.run(event_handle_closure);
}
