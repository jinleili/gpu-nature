use super::{BendingConstraintObj, MeshColoringObj};

// 计算弯曲约束
// 按横向遍历再纵向遍历创建约束，可避免重复
// 所有row | column 约束分别分为两组，共4组
pub fn cal_bend_constraints2(
    horizontal_num: usize, vertical_num: usize,
) -> (Vec<MeshColoringObj>, Vec<BendingConstraintObj>, Vec<[i32; 3]>) {
    let mut bendings: Vec<BendingConstraintObj> =
        Vec::with_capacity(horizontal_num * vertical_num * 3);
    let mut reorder_bendings: Vec<[i32; 3]> = vec![];

    let mut four_groups: Vec<Vec<[i32; 3]>> = vec![vec![], vec![], vec![], vec![], vec![], vec![]];
    let h0: f32 = 0.0;

    // 先计算 row 向约束
    for h in 0..vertical_num {
        let mut offset = 1;
        for w in 0..(horizontal_num / 2) {
            let new_w = w + offset;
            if new_w < (horizontal_num - 1) {
                let v = (h * horizontal_num + new_w) as i32;
                let b0 = v - 1;
                let b1 = v + 1;
                four_groups[offset % 2].push([bendings.len() as i32, -1, -1]);
                bendings.push(BendingConstraintObj { v, b0, b1, h0 });
                offset += 1;
            }
        }
    }
    // 再计算 column 向的两组约束
    for w in 0..horizontal_num {
        let mut offset = 1;
        for h in 0..(vertical_num / 2) {
            let new_h = h + offset;
            if new_h < (vertical_num - 1) {
                let v = (new_h * horizontal_num + w) as i32;
                // 垂直约束
                let b0 = v - horizontal_num as i32;
                let b1 = v + horizontal_num as i32;
                four_groups[2 + offset % 2].push([bendings.len() as i32, -1, -1]);
                bendings.push(BendingConstraintObj { v, b0, b1, h0 });
                offset += 1;
            }
        }
    }
    // 斜向约束
    let mut h = 1;
    let mut offset = 0;
    while h < (vertical_num - 1) {
        let mut w = 1;
        while w < (horizontal_num - 1) {
            let v = (h * horizontal_num + w) as i32;
            let b0 = v + 1 + horizontal_num as i32;
            let b1 = v - 1 - horizontal_num as i32;
            four_groups[4 + offset % 2].push([bendings.len() as i32, -1, -1]);
            bendings.push(BendingConstraintObj { v, b0, b1, h0 });
            w += 2;
        }
        h += 2;
        offset += 1;
    }

    let mut mesh_colorings: Vec<MeshColoringObj> = vec![];
    let mut offset = 0;
    for a_group in four_groups.iter_mut() {
        println!("bend group len: {}", a_group.len());
        let group_len = a_group.len() as u32;
        mesh_colorings.push(MeshColoringObj {
            offset,
            max_num_x: 0,
            max_num_y: 0,
            group_len,
            thread_group: (((group_len + 31) as f32 / 32.0).floor() as u32, 1),
        });
        reorder_bendings.append(a_group);
        offset += group_len;
    }
    (mesh_colorings, bendings, reorder_bendings)
}
