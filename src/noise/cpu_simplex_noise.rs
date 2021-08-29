#[derive(Debug)]
struct Grad {
    x: f64,
    y: f64,
    z: f64,
}

impl Grad {
    pub fn new(x: i32, y: i32, z: i32) -> Grad {
        Grad { x: x as f64, y: y as f64, z: z as f64 }
    }

    pub fn dot2(&self, x: f64, y: f64) -> f64 {
        self.x * x + self.y * y
    }

    pub fn dot3(&self, x: f64, y: f64, z: f64) -> f64 {
        self.x * x + self.y * y + self.z * z
    }
}

pub struct SimplexNoise {
    grad3: Vec<Grad>,
    f2: f64,
    g2: f64,
    perm: Vec<i32>,
    grad_p: Vec<i32>,
}

impl SimplexNoise {
    // 长宽高为 2 且中心点在 （0， 0， 0）位置的立方体
    // 取通过坐标原点且平行于 3 个轴的的面与立方体边的交点：
    // 实质上就是每条边的中心点
    pub fn new() -> SimplexNoise {
        let grad3: Vec<Grad> = vec![
            Grad::new(1, 1, 0),
            Grad::new(-1, 1, 0),
            Grad::new(1, -1, 0),
            Grad::new(-1, -1, 0),
            Grad::new(1, 0, 1),
            Grad::new(-1, 0, 1),
            Grad::new(1, 0, -1),
            Grad::new(-1, 0, -1),
            Grad::new(0, 1, 1),
            Grad::new(0, -1, 1),
            Grad::new(0, 1, -1),
            Grad::new(0, -1, -1),
        ];
        let mut perm: Vec<i32> = Vec::new();
        let mut grad_p: Vec<i32> = Vec::new();
        for i in 0..512 {
            let value = super::P[i & 255];
            perm.push(value);
            grad_p.push(value % 12);
        }

        SimplexNoise {
            grad3: grad3,
            f2: (3.0_f64.sqrt() - 1.) * 0.5,
            g2: (3.0 - 3.0_f64.sqrt()) / 6.0,
            perm: perm,
            grad_p: grad_p,
        }
    }

    pub fn step(&self, xin: f64, yin: f64) -> f64 {
        // 扭曲输入坐标空间，判断它落在哪个 simplex 单元里
        // Hairy factor for 2D
        let s = (xin + yin) * self.f2;
        let i = (xin + s).floor();
        let j = (yin + s).floor();
        let t = (i + j) * self.g2;
        // println!("s: {:?}, {:?}, {:?}, {:?}", s, i, j, t);

        // The x,y distances from the cell origin, unskewed.
        let x0 = xin - i + t;
        let y0 = yin - j + t;

        // For the 2D case, the simplex shape is an equilateral triangle.
        // Determine which simplex we are in.
        // Offsets for second (middle) corner of simplex in (i,j) coords
        let i1: i32;
        let j1: i32;
        if (x0 > y0) {
            // lower triangle, XY order: (0,0)->(1,0)->(1,1)
            i1 = 1;
            j1 = 0;
        } else {
            // upper triangle, YX order: (0,0)->(0,1)->(1,1)
            i1 = 0;
            j1 = 1;
        }
        // println!("i1: {:?}, {:?}, {:?}, {:?}", i1, j1, x0, y0);

        // A step of (1,0) in (i,j) means a step of (1-c,-c) in (x,y), and
        // a step of (0,1) in (i,j) means a step of (-c,1-c) in (x,y), where
        // c = (3-sqrt(3))/6
        // Offsets for middle corner in (x,y) unskewed coords
        let x1 = x0 - i1 as f64 + self.g2;
        let y1 = y0 - j1 as f64 + self.g2;
        // Offsets for last corner in (x,y) unskewed coords
        let x2 = x0 - 1. + 2. * self.g2;
        let y2 = y0 - 1. + 2. * self.g2;
        // Work out the hashed gradient indices of the three simplex corners
        let i = (i as i32) & 255;
        let j = (j as i32) & 255;

        // println!("{:?}, {:?}, {:?}, {:?}", x1, x2, y1, y2);

        let gi0 = &self.grad3[self.grad_p[(i + self.perm[j as usize]) as usize] as usize];
        let gi1 =
            &self.grad3[self.grad_p[(i + i1 + self.perm[(j + j1) as usize]) as usize] as usize];
        let gi2 = &self.grad3[self.grad_p[(i + 1 + self.perm[(j + 1) as usize]) as usize] as usize];
        // println!("{:?}", gi0);
        // println!("{:?}", gi1);
        // println!("{:?}", gi2);

        // 三个相关角的贡献值
        let mut n0: f64 = 0.;
        let mut n1: f64 = 0.;
        let mut n2: f64 = 0.;
        // Calculate the contribution from the three corners
        let mut t0 = 0.5 - x0 * x0 - y0 * y0;
        if t0 < 0. {
            n0 = 0.;
        } else {
            t0 *= t0;
            n0 = t0 * t0 * gi0.dot2(x0, y0); // (x,y) of grad3 used for 2D gradient
        }
        let mut t1 = 0.5 - x1 * x1 - y1 * y1;
        if t1 < 0. {
            n1 = 0.;
        } else {
            t1 *= t1;
            n1 = t1 * t1 * gi1.dot2(x1, y1);
        }
        let mut t2 = 0.5 - x2 * x2 - y2 * y2;
        if t2 < 0. {
            n2 = 0.;
        } else {
            t2 *= t2;
            n2 = t2 * t2 * gi2.dot2(x2, y2);
        }

        // println!(".............");
        // println!("{:?}, {:?}, {:?}", n0, n1, n2);

        // Add contributions from each corner to get the final noise value.
        // The result is scaled to return values in the interval [-1,1].
        70. * (n0 + n1 + n2)
    }
}
