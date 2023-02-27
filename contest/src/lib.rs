pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub trait RatingChanger {
    fn rating(&mut self) -> &mut i32;
}

fn calc_percentage<T: RatingChanger>(p1: i32, p2: i32) -> f64 {
    1f64 / (1f64 + 10f64.powf( ( (p2 - p1) as f64)/(400f64) ) )
}

fn calc<T: RatingChanger>(contestants: &mut Vec<T>,pi: i32,i:usize) -> f64 {
    let mut sum:f64 = 0.0;
    for j in 0..contestants.len(){
        if i == j {
           continue;
        } else {
            sum += calc_percentage::<T>(*contestants[j].rating() , pi);
        }
    }
    sum
}

/// k 表示变化力度（得分变化为默认值的 k 倍）
/// 假设 rank = 下标+1
/// minrating = -10000 ,maxrating = 10000

pub fn modify_rating<T: RatingChanger>(contestants: &mut Vec<T>, k: f64) {
    let mut seed: Vec<f64> = Vec::new();
    for i in 0..contestants.len(){
        let num = *contestants[i].rating();
        let mut sum:f64 = calc(contestants,num,i);
        sum = sum + 1f64;
        seed.push(sum);
    }
    let mut dir: Vec<i32> = Vec::new();
    let mut totdir = 0;
    for i in 0..contestants.len(){
        let mi = (seed[i]*((i+1) as f64)).sqrt();
        let mut l:i32 = -10000;
        let mut r:i32 = 10000;
        while r > l + 1 {
            let mid = (l+r)/2;
            //println!("{} {}",mid,calc(contestants,mid,i)+1.0);
            if calc(contestants,mid,i)+1.0 >= mi {
                l = mid;
            }else{
                r = mid;
            }
        }
        let nowdir = (l-*contestants[i].rating())/2;
        //println!("{} {} {}",seed[i],mi,nowdir);
        dir.push(nowdir);
        totdir += nowdir;
    }
    let inc = (0f64).min( (-10f64).max( -(totdir as f64)/(contestants.len() as f64) ) );
    for i in 0..contestants.len(){
        *contestants[i].rating() += (((dir[i] as f64) + inc) * k) as i32; 
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        struct Users{
            rate:i32
        }
        impl RatingChanger for Users{
            fn rating(&mut self) -> &mut i32 {
                &mut self.rate
            }
        }
        let mut users:Vec<Users> = Vec::new();
        for i in 0..50 {
            let person = Users{
                rate : 2500-i*10
            };
            users.push(person);
        }
        for i in 0..50 {
            println!("{}",users[i].rate);
        }
        modify_rating(&mut users, 1.0);
        for i in 0..50 {
            println!("{}",users[i].rate);
        }
    }
}
