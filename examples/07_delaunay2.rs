use del_gl::gl as gl;

fn resampling_loops(
    loop2vtx_idx: &mut Vec<usize>,
    loop2vtx: &mut Vec<usize>,
    vtx2xy: &mut Vec<nalgebra::Vector2<f32>>,
    max_edge_length: f32) {
    assert_eq!(vtx2xy.len(), loop2vtx.len());
    let loop2vtx_idx0 = loop2vtx_idx.clone();
    let loop2vtx0 = loop2vtx.clone();
    assert!(loop2vtx0.len() >= 2);
    let nloop = loop2vtx_idx0.len() - 1;
    let mut point_idxs_in_edges = vec!(Vec::<usize>::new(); loop2vtx0.len());

    for iloop in 0..nloop {
        assert!(loop2vtx_idx0[iloop + 1] > loop2vtx_idx0[iloop]);
        let np = loop2vtx_idx0[iloop + 1] - loop2vtx_idx0[iloop];
        for ip in 0..np {
            let iipo0 = loop2vtx_idx0[iloop] + (ip + 0) % np;
            let iipo1 = loop2vtx_idx0[iloop] + (ip + 1) % np;
            assert!(iipo0 < loop2vtx0.len());
            assert!(iipo1 < loop2vtx0.len());
            let ipo0 = loop2vtx0[iipo0];
            let ipo1 = loop2vtx0[iipo1];
            assert!(ipo0 < vtx2xy.len());
            assert!(ipo1 < vtx2xy.len());
            let po0 = vtx2xy[ipo0].clone(); // never use reference here because aVec2 will resize afterward
            let po1 = vtx2xy[ipo1].clone(); // never use reference here because aVec2 will resize afterward
            let nadd = ((po0 - po1).norm() / max_edge_length) as i32;
            if nadd == 0 {
                continue;
            }
            for iadd in 0..nadd {
                let r2 = (iadd + 1) as f32 / (nadd + 1) as f32;
                let v2 = (1. - r2) * po0 + r2 * po1;
                let ipo2 = vtx2xy.len();
                vtx2xy.push(v2);
                assert!(iipo0 < point_idxs_in_edges.len());
                point_idxs_in_edges[iipo0].push(ipo2);
            }
        }
    }
    //
    loop2vtx_idx.resize(nloop + 1, usize::MAX);
    loop2vtx_idx[0] = 0;
    for iloop in 0..nloop {
        let nbar0 = loop2vtx_idx0[iloop + 1] - loop2vtx_idx0[iloop];
        let mut nbar1 = nbar0;
        for ibar in 0..nbar0 {
            let iip_loop = loop2vtx_idx0[iloop] + ibar;
            nbar1 += point_idxs_in_edges[iip_loop].len();
        }
        loop2vtx_idx[iloop + 1] = loop2vtx_idx[iloop] + nbar1;
    }
    // adding new vertices on the outline
    loop2vtx.resize(loop2vtx_idx[nloop], usize::MAX);
    let mut ivtx0 = 0;
    for iloop in 0..nloop {
        for iip_loop in loop2vtx_idx0[iloop]..loop2vtx_idx0[iloop + 1] {
            let ip_loop = loop2vtx0[iip_loop];
            loop2vtx[ivtx0] = ip_loop;
            ivtx0 += 1;
            for iadd in 0..point_idxs_in_edges[ip_loop].len() {
                loop2vtx[ivtx0] = point_idxs_in_edges[iip_loop][iadd];
                ivtx0 += 1;
            }
        }
    }
    assert_eq!(loop2vtx.len(), vtx2xy.len());
    assert_eq!(loop2vtx.len(), ivtx0);
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
        let mut loop2vtx_idx = vec!(0, 4);
        let mut loop2vtx = vec!(0, 1, 2, 3);
        vtx2xy.push(nalgebra::Vector2::<f32>::new(-1.0, -1.0));
        vtx2xy.push(nalgebra::Vector2::<f32>::new(1.0, -1.0));
        vtx2xy.push(nalgebra::Vector2::<f32>::new(1.0, 1.0));
        vtx2xy.push(nalgebra::Vector2::<f32>::new(-1.0, 1.0));
        resampling_loops(
            &mut loop2vtx_idx, &mut loop2vtx, &mut vtx2xy,
            0.12);
        meshing_single_connected_shape2(
            &mut vtx2tri, &mut vtx2xy, &mut tri2vtx,
            &loop2vtx_idx, &loop2vtx);
        let mut vtx2flg = vec!(0; vtx2tri.len());
        let mut tri2flg = vec!(0; tri2vtx.len());
        let nvtx0 = vtx2xy.len();
        meshing_inside(
            &mut vtx2tri, &mut tri2vtx, &mut vtx2xy,
            &mut vtx2flg, &mut tri2flg,
            nvtx0, 0, 0.12);
    }

    let (mut viewer, event_loop) = del_gl::glutin::viewer2::Viewer2::open();

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
            let line_vtx: Vec<usize> = del_msh::line2vtx::from_epecific_edges_of_uniform_mesh(
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
