use del_gl::gl as gl;

fn resampling_loops(
    loop2idx: &mut Vec<usize>,
    idx2vtx: &mut Vec<usize>,
    vtx2xy: &mut Vec<nalgebra::Vector2<f32>>,
    max_edge_length: f32) {
    assert_eq!(vtx2xy.len(), idx2vtx.len());
    let loop2idx_old = loop2idx.clone();
    let idx2vtx_old = idx2vtx.clone();
    assert!(idx2vtx_old.len() >= 2);
    let num_loop = loop2idx_old.len() - 1;
    let mut point_idxs_in_edges = vec!(Vec::<usize>::new(); idx2vtx_old.len());

    for i_loop in 0..num_loop {
        assert!(loop2idx_old[i_loop + 1] > loop2idx_old[i_loop]);
        let num_node = loop2idx_old[i_loop + 1] - loop2idx_old[i_loop];
        for i_node in 0..num_node {
            let idx0 = loop2idx_old[i_loop] + (i_node + 0) % num_node;
            let idx1 = loop2idx_old[i_loop] + (i_node + 1) % num_node;
            assert!(idx0 < idx2vtx_old.len());
            assert!(idx1 < idx2vtx_old.len());
            let i_vtx0 = idx2vtx_old[idx0];
            let i_vtx1 = idx2vtx_old[idx1];
            assert!(i_vtx0 < vtx2xy.len());
            assert!(i_vtx1 < vtx2xy.len());
            let p0 = vtx2xy[i_vtx0].clone(); // never use reference here because aVec2 will resize afterward
            let p1 = vtx2xy[i_vtx1].clone(); // never use reference here because aVec2 will resize afterward
            let num_vtx_to_add = ((p0 - p1).norm() / max_edge_length) as i32;
            if num_vtx_to_add == 0 {
                continue;
            }
            for iadd in 0..num_vtx_to_add {
                let r2 = (iadd + 1) as f32 / (num_vtx_to_add + 1) as f32;
                let v2 = (1. - r2) * p0 + r2 * p1;
                let ipo2 = vtx2xy.len();
                vtx2xy.push(v2);
                assert!(idx0 < point_idxs_in_edges.len());
                point_idxs_in_edges[idx0].push(ipo2);
            }
        }
    }
    //
    loop2idx.resize(num_loop + 1, usize::MAX);
    loop2idx[0] = 0;
    for i_loop in 0..num_loop {
        let num_edge_old = loop2idx_old[i_loop + 1] - loop2idx_old[i_loop];
        let mut num_edge_new = num_edge_old;
        for i_edge_old in 0..num_edge_old {
            let iip_loop = loop2idx_old[i_loop] + i_edge_old;
            num_edge_new += point_idxs_in_edges[iip_loop].len();
        }
        loop2idx[i_loop + 1] = loop2idx[i_loop] + num_edge_new;
    }
    // adding new vertices on the outline
    idx2vtx.resize(loop2idx[num_loop], usize::MAX);
    let mut i_vtx0 = 0;
    for i_loop in 0..num_loop {
        for iip_loop in loop2idx_old[i_loop]..loop2idx_old[i_loop + 1] {
            let ip_loop = idx2vtx_old[iip_loop];
            idx2vtx[i_vtx0] = ip_loop;
            i_vtx0 += 1;
            for iadd in 0..point_idxs_in_edges[ip_loop].len() {
                idx2vtx[i_vtx0] = point_idxs_in_edges[iip_loop][iadd];
                i_vtx0 += 1;
            }
        }
    }
    assert_eq!(idx2vtx.len(), vtx2xy.len());
    assert_eq!(idx2vtx.len(), i_vtx0);
}

fn main() {
    use del_dtri::topology::{
        DynamicTriangle,
        DynamicVertex};
    use del_dtri::mesher2::meshing_single_connected_shape2;
    use del_dtri::mesher2::meshing_inside;
    let mut tri2vtx = Vec::<DynamicTriangle>::new();
    let mut vtx2tri = Vec::<DynamicVertex>::new();
    let mut vtx2xy = Vec::<nalgebra::Vector2<f32>>::new();
    {
        let mut loop2idx = vec!(0, 4);
        let mut idx2vtx = vec!(0, 1, 2, 3);
        vtx2xy.push(nalgebra::Vector2::<f32>::new(-1.0, -1.0));
        vtx2xy.push(nalgebra::Vector2::<f32>::new(1.0, -1.0));
        vtx2xy.push(nalgebra::Vector2::<f32>::new(1.0, 1.0));
        vtx2xy.push(nalgebra::Vector2::<f32>::new(-1.0, 1.0));
        resampling_loops(
            &mut loop2idx, &mut idx2vtx, &mut vtx2xy,
            0.12);
        meshing_single_connected_shape2(
            &mut vtx2tri, &mut vtx2xy, &mut tri2vtx,
            &loop2idx, &idx2vtx);
        let mut vtx2flg = vec!(0; vtx2tri.len());
        let mut tri2flg = vec!(0; tri2vtx.len());
        let num_vtx = vtx2xy.len();
        meshing_inside(
            &mut vtx2tri, &mut tri2vtx, &mut vtx2xy,
            &mut vtx2flg, &mut tri2flg,
            num_vtx, 0, 0.12_f32);
    }

    let (mut viewer, event_loop) = del_gl::glutin::viewer2::Viewer2::open();
    println!("OpenGL Version: {}", viewer.get_opengl_version());
    println!("Pixel format:: {:?}", viewer.windowed_context.get_pixel_format());

    let mut drawer = del_gl::mesh::Drawer::new();
    {
        drawer.compile_shader(&viewer.gl);
        {
            let mut vtx_xyz = Vec::<f32>::new();
            for p in &vtx2xy {
                vtx_xyz.push(p.x);
                vtx_xyz.push(p.y);
            }
            drawer.update_vertex(&viewer.gl, &vtx_xyz, 2);
        }
        let tri_vtx0 = {
            let mut tri_vtx0 = Vec::<usize>::new();
            for t in &tri2vtx {
                tri_vtx0.push(t.v[0]);
                tri_vtx0.push(t.v[1]);
                tri_vtx0.push(t.v[2]);
            }
            tri_vtx0
        };
        drawer.add_element(&viewer.gl, gl::TRIANGLES, &tri_vtx0, [1., 0., 0.]);

        {
            let line_vtx: Vec<usize> = del_msh::line2vtx::from_sepecific_edges_of_uniform_mesh(
                &tri_vtx0, 3,
                &[0, 1, 1, 2, 2, 0],
                vtx2xy.len());
            drawer.add_element(&viewer.gl, gl::LINES, &line_vtx, [0., 0., 0.]);
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
            let mat_projection = viewer.nav.projection_matrix(viewer.ui_state.win_width, viewer.ui_state.win_height);
            let mat_modelview = viewer.nav.modelview_matrix();
            drawer.draw(
                &viewer.gl,
                mat_modelview.as_slice(), // nalgebra is column major same as OpenGL
                mat_projection.as_slice()); // nalgebra is column major same as OpenGL
            drawer.draw_points(
                &viewer.gl,
                mat_modelview.as_slice(), // nalgebra is column major same as OpenGL
                mat_projection.as_slice()); // nalgebra is column major same as OpenGL
            viewer.windowed_context.swap_buffers().unwrap();
        }
    };
    event_loop.run(event_handle_closure);
}
