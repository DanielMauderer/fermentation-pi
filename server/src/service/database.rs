pub mod project {
    use std::{
        fs::File,
        io::{Read, Write},
        time::SystemTime,
    };

    use serde::{Deserialize, Serialize};

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Project {
        pub id: u32,
        pub name: String,
        pub description: String,
        pub created_at: u64,
        pub start_at: Option<u64>,
        pub endend_at: Option<u64>,
    }

    /* read project json path: db/projects.json */
    pub fn read_projects() -> Vec<Project> {
        let mut file = File::open("./db/projects.json").unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();
        let projects: Vec<Project> = serde_json::from_str(&data).unwrap();
        projects
    }

    pub fn read_project(id: u32) -> Project {
        let projects = read_projects();
        let project = projects.iter().find(|&p| p.id == id).unwrap();
        project.clone()
    }

    pub fn create_new_project(name: String, description: String) -> Project {
        let mut projects = read_projects();
        let id = projects.len() as u32 + 1;
        let created_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let project = Project {
            id,
            name,
            description,
            created_at,
            start_at: None,
            endend_at: None,
        };
        projects.push(project.clone());
        let mut file = File::create("./db/projects.json").unwrap();
        file.write_all(serde_json::to_string(&projects).unwrap().as_bytes())
            .unwrap();
        project
    }

    pub fn update_project(id: u32, name: Option<String>, description: Option<String>) {
        let mut projects = read_projects();
        let project = projects.iter_mut().find(|p| p.id == id).unwrap();
        if name.is_some() {
            project.name = name.unwrap();
        }
        if description.is_some() {
            project.description = description.unwrap();
        }
        let mut file = File::create("./db/projects.json").unwrap();
        file.write_all(serde_json::to_string(&projects).unwrap().as_bytes())
            .unwrap();
    }

    pub fn delete_project(id: u32) {
        let mut projects = read_projects();
        let index = match projects.iter().position(|p| p.id == id) {
            Some(index) => index,
            None => return,
        };
        projects.remove(index);
        let mut file = File::create("./db/projects.json").unwrap();
        file.write_all(serde_json::to_string(&projects).unwrap().as_bytes())
            .unwrap();
    }

    pub fn start_project(id: u32, start_at: u64) {
        let mut projects = read_projects();
        if check_if_project_is_running(&projects) {
            return;
        }
        let project = projects.iter_mut().find(|p| p.id == id).unwrap();

        if project.start_at.is_some() {
            return;
        }

        project.start_at = Some(start_at);
        let mut file = File::create("./db/projects.json").unwrap();
        file.write_all(serde_json::to_string(&projects).unwrap().as_bytes())
            .unwrap();
    }

    pub fn end_project(id: u32, endend_at: u64) {
        let mut projects = read_projects();
        let project = projects.iter_mut().find(|p| p.id == id).unwrap();
        project.endend_at = Some(endend_at);
        let mut file = File::create("./db/projects.json").unwrap();
        file.write_all(serde_json::to_string(&projects).unwrap().as_bytes())
            .unwrap();
    }

    pub fn get_project(id: u32) -> Project {
        let projects = read_projects();
        let project = projects.iter().find(|&p| p.id == id).unwrap();
        project.clone()
    }

    fn check_if_project_is_running(projects: &Vec<Project>) -> bool {
        for project in projects {
            if project.start_at.is_some() && project.endend_at.is_none() {
                return true;
            }
        }
        false
    }
}

pub mod sensor {
    use std::{
        fs::File,
        io::{Read, Write},
        path::Path,
    };

    use ::serde::{Serialize, Serializer};
    use serde::Deserialize;

    #[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
    #[serde(crate = "rocket::serde")]
    pub struct SensorData {
        #[serde(serialize_with = "round_serialize")]
        pub temp: f32,
        #[serde(serialize_with = "round_serialize")]
        pub hum: f32,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
    #[serde(crate = "rocket::serde")]
    pub struct HistoricSensorData {
        pub time: u64,
        pub data: SensorData,
    }

    fn round_serialize<S>(x: &f32, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_f32((x * 100.0).round() / 100.0)
    }

    pub fn get_all_data(time: u64) -> Vec<HistoricSensorData> {
        let historic_sensor_data: Vec<HistoricSensorData> = Vec::new();
        let date = chrono::NaiveDateTime::from_timestamp_opt(time as i64, 0);

        create_new_sensor_page(time);

        match date {
            Some(date) => {
                let path = format!("./db/sensor/{}.json", date.format("%Y-%m-%d"));
                let mut file = File::open(path).unwrap();
                let mut data = String::new();
                file.read_to_string(&mut data).unwrap();
                let historic_data: Vec<HistoricSensorData> = serde_json::from_str(&data).unwrap();
                return historic_data;
            }
            None => {}
        }
        historic_sensor_data
    }

    pub fn add_datapoint(data: HistoricSensorData) {
        let date = chrono::NaiveDateTime::from_timestamp_opt(data.time as i64, 0);
        match date {
            Some(date) => {
                let path = format!("./db/sensor/{}.json", date.format("%Y-%m-%d"));
                let mut historic_data = get_all_data(data.time);
                historic_data.push(data);
                let mut file = File::create(path).unwrap();
                file.write_all(serde_json::to_string(&historic_data).unwrap().as_bytes())
                    .unwrap();
            }
            None => {}
        }
    }

    fn create_new_sensor_page(time: u64) {
        let date = chrono::NaiveDateTime::from_timestamp_opt(time as i64, 0);
        match date {
            Some(date) => {
                let path = format!("./db/sensor/{}.json", date.format("%Y-%m-%d"));
                if Path::new(&path).exists() {
                    return;
                }
                let mut file = File::create(path).unwrap();
                file.write_all("[]".as_bytes()).unwrap();
            }
            None => {}
        }
    }
}
