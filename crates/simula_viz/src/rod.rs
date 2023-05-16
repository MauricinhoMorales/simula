use bevy::math::{Vec2, Vec3};
use bevy::render::{
    mesh::{Indices, Mesh},
    render_resource::PrimitiveTopology,
};
use simula_core::{
    ease::{Ease, EaseFunction},
    lerp::lerp,
};

fn interpolate(ease_func: EaseFunction, a: f32, b: f32, t: f32) -> f32 {
    let t = t.calc(ease_func);
    lerp(&a, &b, &t)
}

/// A cylinder with variable hemispheres at the top and bottom
#[derive(Debug, Copy, Clone)]
pub struct Rod {
    /// Interpolation function to go from north to south
    pub ease_func: EaseFunction,
    /// North radius on the xz plane.
    pub north_radius: f32,
    /// South radius on the xz plane.
    pub south_radius: f32,
    /// Number of sections in cylinder between hemispheres.
    pub rings: usize,
    /// Height of the middle cylinder on the y axis, excluding the hemispheres.
    pub depth: f32,
    /// Number of latitudes, distributed by inclination. Must be even.
    pub latitudes: usize,
    /// Number of longitudes, or meridians, distributed by azimuth.
    pub longitudes: usize,
    /// Manner in which UV coordinates are distributed vertically.
    pub uv_profile: RodUvProfile,
}

impl Default for Rod {
    fn default() -> Self {
        Rod {
            ease_func: EaseFunction::Linear,
            north_radius: 0.5,
            south_radius: 0.8,
            rings: 10,
            depth: 1.0,
            latitudes: 16,
            longitudes: 32,
            uv_profile: RodUvProfile::Uniform,
        }
    }
}

impl From<Rod> for Mesh {
    fn from(value: Rod) -> Self {
        let rod_mesh = RodMesh::from(value);
        rod_mesh.mesh
    }
}

#[derive(Debug, Clone, Copy)]
/// Manner in which UV coordinates are distributed vertically.
pub enum RodUvProfile {
    /// UV space is distributed by how much of the capsule consists of the hemispheres.
    Aspect,
    /// Hemispheres get UV space according to the ratio of latitudes to rings.
    Uniform,
    /// Upper third of the texture goes to the northern hemisphere, middle third to the cylinder
    /// and lower third to the southern one.
    Fixed,
}

impl Default for RodUvProfile {
    fn default() -> Self {
        RodUvProfile::Aspect
    }
}

pub struct RodMesh {
    pub rod: Rod,
    mesh: Mesh,
    pub vs: Vec<[f32; 3]>,
    pub vns: Vec<[f32; 3]>,
    pub vts: Vec<[f32; 2]>,
    pub vxs: Vec<[f32; 4]>,
    pub tris: Vec<u32>,
}

impl From<Rod> for RodMesh {
    #[allow(clippy::needless_range_loop)]
    fn from(rod: Rod) -> Self {
        // code adapted from https://behreajj.medium.com/making-a-capsule-mesh-via-script-in-five-3d-environments-c2214abf02db

        let Rod {
            ease_func,
            north_radius,
            south_radius,
            rings,
            depth,
            latitudes,
            longitudes,
            uv_profile,
        } = rod;

        let radius = 1.0;
        let calc_middle = rings > 0;
        let half_lats = latitudes / 2;
        let half_latsn1 = half_lats - 1;
        let half_latsn2 = half_lats - 2;
        let ringsp1 = rings + 1;
        let lonsp1 = longitudes + 1;
        let half_depth = depth * 0.5;
        let north_summit = half_depth + north_radius;
        let south_summit = half_depth + south_radius;

        // Vertex index offsets.
        let vert_offset_north_hemi = longitudes;
        let vert_offset_north_equator = vert_offset_north_hemi + lonsp1 * half_latsn1;
        let vert_offset_cylinder = vert_offset_north_equator + lonsp1;
        let vert_offset_south_equator = if calc_middle {
            vert_offset_cylinder + lonsp1 * rings
        } else {
            vert_offset_cylinder
        };
        let vert_offset_south_hemi = vert_offset_south_equator + lonsp1;
        let vert_offset_south_polar = vert_offset_south_hemi + lonsp1 * half_latsn2;
        let vert_offset_south_cap = vert_offset_south_polar + lonsp1;

        // Initialize arrays.
        let vert_len = vert_offset_south_cap + longitudes;

        let mut vs: Vec<Vec3> = vec![Vec3::ZERO; vert_len];
        let mut vts: Vec<Vec2> = vec![Vec2::ZERO; vert_len];
        let mut vns: Vec<Vec3> = vec![Vec3::ZERO; vert_len];

        let to_theta = 2.0 * std::f32::consts::PI / longitudes as f32;
        let to_phi = std::f32::consts::PI / latitudes as f32;
        let to_tex_horizontal = 1.0 / longitudes as f32;
        let to_tex_vertical = 1.0 / half_lats as f32;

        let vt_aspect_ratio = match uv_profile {
            RodUvProfile::Aspect => radius / (depth + radius + radius),
            RodUvProfile::Uniform => half_lats as f32 / (ringsp1 + latitudes) as f32,
            RodUvProfile::Fixed => 1.0 / 3.0,
        };
        let vt_aspect_north = 1.0 - vt_aspect_ratio;
        let vt_aspect_south = vt_aspect_ratio;

        let mut theta_cartesian: Vec<Vec2> = vec![Vec2::ZERO; longitudes];
        let mut rho_theta_cartesian: Vec<Vec2> = vec![Vec2::ZERO; longitudes];
        let mut s_texture_cache: Vec<f32> = vec![0.0; lonsp1];

        for j in 0..longitudes {
            let jf = j as f32;
            let s_texture_polar = 1.0 - ((jf + 0.5) * to_tex_horizontal);
            let theta = jf * to_theta;

            let cos_theta = theta.cos();
            let sin_theta = theta.sin();

            theta_cartesian[j] = Vec2::new(cos_theta, sin_theta);
            rho_theta_cartesian[j] = Vec2::new(radius * cos_theta, radius * sin_theta);

            // North.
            vs[j] = Vec3::new(0.0, north_summit, 0.0);
            vts[j] = Vec2::new(s_texture_polar, 1.0);
            vns[j] = Vec3::new(0.0, 1.0, 0.0);

            // South.
            let idx = vert_offset_south_cap + j;
            vs[idx] = Vec3::new(0.0, -south_summit, 0.0);
            vts[idx] = Vec2::new(s_texture_polar, 0.0);
            vns[idx] = Vec3::new(0.0, -1.0, 0.0);
        }

        // Equatorial vertices.
        for j in 0..lonsp1 {
            let s_texture = 1.0 - j as f32 * to_tex_horizontal;
            s_texture_cache[j] = s_texture;

            // Wrap to first element upon reaching last.
            let j_mod = j % longitudes;
            let tc = theta_cartesian[j_mod];
            let rtc = rho_theta_cartesian[j_mod];

            // North equator.
            let idxn = vert_offset_north_equator + j;
            vs[idxn] = Vec3::new(rtc.x * north_radius, half_depth, -rtc.y * north_radius);
            vts[idxn] = Vec2::new(s_texture, vt_aspect_north);
            vns[idxn] = Vec3::new(tc.x, 0.0, -tc.y);

            // South equator.
            let idxs = vert_offset_south_equator + j;
            vs[idxs] = Vec3::new(rtc.x * south_radius, -half_depth, -rtc.y * south_radius);
            vts[idxs] = Vec2::new(s_texture, vt_aspect_south);
            vns[idxs] = Vec3::new(tc.x, 0.0, -tc.y);
        }

        // Hemisphere vertices.
        for i in 0..half_latsn1 {
            let ip1f = i as f32 + 1.0;
            let phi = ip1f * to_phi;

            // For coordinates.
            let cos_phi_south = phi.cos();
            let sin_phi_south = phi.sin();

            let cos_phi_north = phi.sin();
            let sin_phi_north = -phi.cos();

            let rho_cos_phi_north = radius * cos_phi_north;
            let rho_sin_phi_north = radius * sin_phi_north;
            let z_offset_north = half_depth - rho_sin_phi_north * north_radius;

            let rho_cos_phi_south = radius * cos_phi_south;
            let rho_sin_phi_south = radius * sin_phi_south;
            let z_offset_south = -half_depth - rho_sin_phi_south * south_radius;

            // For texture coordinates.
            let t_tex_fac = ip1f * to_tex_vertical;
            let cmpl_tex_fac = 1.0 - t_tex_fac;
            let t_tex_north = cmpl_tex_fac + vt_aspect_north * t_tex_fac;
            let t_tex_south = cmpl_tex_fac * vt_aspect_south;

            let i_lonsp1 = i * lonsp1;
            let vert_curr_lat_north = vert_offset_north_hemi + i_lonsp1;
            let vert_curr_lat_south = vert_offset_south_hemi + i_lonsp1;

            for j in 0..lonsp1 {
                let j_mod = j % longitudes;

                let s_texture = s_texture_cache[j];
                let tc = theta_cartesian[j_mod];

                let north_radius_weight = interpolate(
                    ease_func,
                    south_radius,
                    north_radius,
                    (z_offset_north + half_depth) / depth,
                );
                let south_radius_weight = interpolate(
                    ease_func,
                    south_radius,
                    north_radius,
                    (z_offset_south + half_depth) / depth,
                );

                // North hemisphere.
                let idxn = vert_curr_lat_north + j;
                vs[idxn] = Vec3::new(
                    rho_cos_phi_north * tc.x * north_radius_weight,
                    z_offset_north,
                    -rho_cos_phi_north * tc.y * north_radius_weight,
                );
                vts[idxn] = Vec2::new(s_texture, t_tex_north);
                vns[idxn] = Vec3::new(cos_phi_north * tc.x, -sin_phi_north, -cos_phi_north * tc.y);

                // South hemisphere.
                let idxs = vert_curr_lat_south + j;
                vs[idxs] = Vec3::new(
                    rho_cos_phi_south * tc.x * south_radius_weight,
                    z_offset_south,
                    -rho_cos_phi_south * tc.y * south_radius_weight,
                );
                vts[idxs] = Vec2::new(s_texture, t_tex_south);
                vns[idxs] = Vec3::new(cos_phi_south * tc.x, -sin_phi_south, -cos_phi_south * tc.y);
            }
        }

        // Cylinder vertices.
        if calc_middle {
            // Exclude both origin and destination edges
            // (North and South equators) from the interpolation.
            let to_fac = 1.0 / ringsp1 as f32;
            let mut idx_cyl_lat = vert_offset_cylinder;

            for h in 1..ringsp1 {
                let fac = h as f32 * to_fac;
                let cmpl_fac = 1.0 - fac;
                let t_texture = cmpl_fac * vt_aspect_north + fac * vt_aspect_south;
                let z = half_depth - depth * fac;

                let radius_weight = interpolate(
                    ease_func,
                    south_radius,
                    north_radius,
                    (z + half_depth) / depth,
                );

                for j in 0..lonsp1 {
                    let j_mod = j % longitudes;
                    let tc = theta_cartesian[j_mod];
                    let rtc = rho_theta_cartesian[j_mod];
                    let s_texture = s_texture_cache[j];

                    vs[idx_cyl_lat] = Vec3::new(rtc.x * radius_weight, z, -rtc.y * radius_weight);
                    vts[idx_cyl_lat] = Vec2::new(s_texture, t_texture);
                    vns[idx_cyl_lat] = Vec3::new(tc.x, 0.0, -tc.y);

                    idx_cyl_lat += 1;
                }
            }
        }

        // Triangle indices.

        // Stride is 3 for polar triangles;
        // stride is 6 for two triangles forming a quad.
        let lons3 = longitudes * 3;
        let lons6 = longitudes * 6;
        let hemi_lons = half_latsn1 * lons6;

        let tri_offset_north_hemi = lons3;
        let tri_offset_cylinder = tri_offset_north_hemi + hemi_lons;
        let tri_offset_south_hemi = tri_offset_cylinder + ringsp1 * lons6;
        let tri_offset_south_cap = tri_offset_south_hemi + hemi_lons;

        let fs_len = tri_offset_south_cap + lons3;
        let mut tris: Vec<u32> = vec![0; fs_len];

        // Polar caps.
        let mut i = 0;
        let mut k = 0;
        let mut m = tri_offset_south_cap;
        while i < longitudes {
            // North.
            tris[k] = i as u32;
            tris[k + 1] = (vert_offset_north_hemi + i) as u32;
            tris[k + 2] = (vert_offset_north_hemi + i + 1) as u32;

            // South.
            tris[m] = (vert_offset_south_cap + i) as u32;
            tris[m + 1] = (vert_offset_south_polar + i + 1) as u32;
            tris[m + 2] = (vert_offset_south_polar + i) as u32;

            i += 1;
            k += 3;
            m += 3;
        }

        // Hemispheres.

        let mut i = 0;
        let mut k = tri_offset_north_hemi;
        let mut m = tri_offset_south_hemi;

        while i < half_latsn1 {
            let i_lonsp1 = i * lonsp1;

            let vert_curr_lat_north = vert_offset_north_hemi + i_lonsp1;
            let vert_next_lat_north = vert_curr_lat_north + lonsp1;

            let vert_curr_lat_south = vert_offset_south_equator + i_lonsp1;
            let vert_next_lat_south = vert_curr_lat_south + lonsp1;

            let mut j = 0;
            while j < longitudes {
                // North.
                let north00 = vert_curr_lat_north + j;
                let north01 = vert_next_lat_north + j;
                let north11 = vert_next_lat_north + j + 1;
                let north10 = vert_curr_lat_north + j + 1;

                tris[k] = north00 as u32;
                tris[k + 1] = north11 as u32;
                tris[k + 2] = north10 as u32;

                tris[k + 3] = north00 as u32;
                tris[k + 4] = north01 as u32;
                tris[k + 5] = north11 as u32;

                // South.
                let south00 = vert_curr_lat_south + j;
                let south01 = vert_next_lat_south + j;
                let south11 = vert_next_lat_south + j + 1;
                let south10 = vert_curr_lat_south + j + 1;

                tris[m] = south00 as u32;
                tris[m + 1] = south11 as u32;
                tris[m + 2] = south10 as u32;

                tris[m + 3] = south00 as u32;
                tris[m + 4] = south01 as u32;
                tris[m + 5] = south11 as u32;

                j += 1;
                k += 6;
                m += 6;
            }

            i += 1;
        }

        // Cylinder.
        let mut i = 0;
        let mut k = tri_offset_cylinder;

        while i < ringsp1 {
            let vert_curr_lat = vert_offset_north_equator + i * lonsp1;
            let vert_next_lat = vert_curr_lat + lonsp1;

            let mut j = 0;
            while j < longitudes {
                let cy00 = vert_curr_lat + j;
                let cy01 = vert_next_lat + j;
                let cy11 = vert_next_lat + j + 1;
                let cy10 = vert_curr_lat + j + 1;

                tris[k] = cy00 as u32;
                tris[k + 1] = cy11 as u32;
                tris[k + 2] = cy10 as u32;

                tris[k + 3] = cy00 as u32;
                tris[k + 4] = cy01 as u32;
                tris[k + 5] = cy11 as u32;

                j += 1;
                k += 6;
            }

            i += 1;
        }

        let vs: Vec<[f32; 3]> = vs.into_iter().map(Into::into).collect();
        let vns: Vec<[f32; 3]> = vns.into_iter().map(Into::into).collect();
        let vts: Vec<[f32; 2]> = vts.into_iter().map(Into::into).collect();

        assert_eq!(vs.len(), vert_len);
        assert_eq!(tris.len(), fs_len);

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vs.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vns.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vts.clone());
        mesh.set_indices(Some(Indices::U32(tris.clone())));
        mesh.generate_tangents()
            .expect("Failed to generate tangents");

        let mut vxs = vec![];
        vxs.resize(vs.len(), [0f32; 4]);

        RodMesh {
            rod,
            mesh,
            vs,
            vns,
            vts,
            vxs,
            tris,
        }
    }
}
