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
        pub settings: Settings,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Settings {
        pub hum: f32,
        pub temp: f32,
    }

    /* read project json path: db/projects.json */
    pub fn read_projects() -> Result<Vec<Project>, Box<dyn std::error::Error>> {
        let mut file = File::open("./db/projects.json")?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        let projects: Vec<Project> = serde_json::from_str(&data)?;
        Ok(projects)
    }

    pub fn read_project(id: u32) -> Result<Project, Box<dyn std::error::Error>> {
        let projects = read_projects()?;
        let project = match projects.iter().find(|&p| p.id == id) {
            Some(project) => project,
            None => return Err(Box::from("Project not found")),
        };
        Ok(project.clone())
    }

    pub fn create_new_project(
        name: String,
        description: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        write_project(None, |project| {
            project.name = name.clone();
            project.description = description.clone();
            Ok(())
        })?;
        Ok(())
    }

    pub fn update_project(
        id: u32,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        write_project(Some(id), |project| {
            project.name = name.clone().unwrap_or(project.name.clone());
            project.description = description.clone().unwrap_or(project.description.clone());

            Ok(())
        })
    }

    pub fn delete_project(id: u32) -> Result<(), Box<dyn std::error::Error>> {
        let mut projects = read_projects()?;
        let index = match projects.iter().position(|p| p.id == id) {
            Some(index) => index,
            None => return Err(Box::from("Project not found")),
        };
        projects.remove(index);
        let mut file = File::create("./db/projects.json")?;
        file.write_all(serde_json::to_string(&projects)?.as_bytes())?;
        Ok(())
    }

    pub fn start_project(id: u32, start_at: u64) -> Result<(), Box<dyn std::error::Error>> {
        match get_active_project() {
            Ok(project) => {
                if project.id != id {
                    return Err(Box::from("Another project is already running"));
                }
            }
            Err(_) => {}
        }

        write_project(Some(id), |project| {
            project.start_at = Some(start_at);
            Ok(())
        })
    }

    pub fn end_project(id: u32, endend_at: u64) -> Result<(), Box<dyn std::error::Error>> {
        write_project(Some(id), |project| {
            project.endend_at = Some(endend_at);
            Ok(())
        })
    }

    pub fn get_project(id: u32) -> Result<Project, Box<dyn std::error::Error>> {
        let projects = read_projects()?;
        match projects.iter().find(|&p| p.id == id) {
            Some(project) => Ok(project.clone()),
            None => return Err(Box::from("Project not found")),
        }
    }

    pub fn get_active_project() -> Result<Project, Box<dyn std::error::Error>> {
        let projects = read_projects()?;
        match projects
            .iter()
            .find(|&p| p.start_at.is_some() && p.endend_at.is_none())
        {
            Some(project) => Ok(project.clone()),
            None => return Err(Box::from("Project not found")),
        }
    }

    fn write_project(
        id: Option<u32>,
        f: impl Fn(&mut Project) -> Result<(), Box<dyn std::error::Error>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut projects = read_projects()?;
        match id {
            Some(id) => mut_existing_project(&mut projects, id, &f)?,
            None => mut_new_project(&mut projects, f)?,
        };
        let mut file = File::create("./db/projects.json")?;
        file.write_all(serde_json::to_string(&projects)?.as_bytes())?;
        Ok(())
    }

    fn mut_new_project(
        projects: &mut Vec<Project>,
        f: impl Fn(&mut Project) -> Result<(), Box<dyn std::error::Error>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let id = projects.len() as u32 + 1;
        let created_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs();
        let mut project = Project {
            id,
            name: String::from(""),
            description: String::from(""),
            created_at,
            start_at: None,
            endend_at: None,
            settings: Settings {
                hum: 75.0,
                temp: 30.0,
            },
        };
        f(&mut project)?;
        projects.push(project.clone());
        Ok(())
    }

    fn mut_existing_project(
        projects: &mut Vec<Project>,
        id: u32,
        f: &impl Fn(&mut Project) -> Result<(), Box<dyn std::error::Error>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match projects.iter_mut().find(|p| p.id == id) {
            Some(project) => f(project),
            None => return Err(Box::from("Project not found")),
        }
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

    pub fn get_all_data(time: u64) -> Result<Vec<HistoricSensorData>, Box<dyn std::error::Error>> {
        let historic_sensor_data: Vec<HistoricSensorData> = Vec::new();
        let date = chrono::NaiveDateTime::from_timestamp_opt(time as i64, 0);

        create_new_sensor_page(time)?;

        match date {
            Some(date) => {
                let path = format!("./db/sensor/{}.json", date.format("%Y-%m-%d"));
                let mut file = File::open(path)?;
                let mut data = String::new();
                file.read_to_string(&mut data)?;
                let historic_data: Vec<HistoricSensorData> = serde_json::from_str(&data)?;
                return Ok(historic_data);
            }
            None => {}
        }
        Ok(historic_sensor_data)
    }

    pub fn add_datapoint(data: HistoricSensorData) -> Result<(), Box<dyn std::error::Error>> {
        match chrono::NaiveDateTime::from_timestamp_opt(data.time as i64, 0) {
            Some(date) => {
                let path = format!("./db/sensor/{}.json", date.format("%Y-%m-%d"));
                let mut historic_data = get_all_data(data.time)?;
                historic_data.push(data);
                let mut file = File::create(path)?;
                file.write_all(serde_json::to_string(&historic_data)?.as_bytes())?;
            }
            None => {}
        }
        Ok(())
    }

    fn create_new_sensor_page(time: u64) -> Result<(), Box<dyn std::error::Error>> {
        match chrono::NaiveDateTime::from_timestamp_opt(time as i64, 0) {
            Some(date) => {
                let path = format!("./db/sensor/{}.json", date.format("%Y-%m-%d"));
                if Path::new(&path).exists() {
                    return Ok(());
                }
                let mut file = File::create(path)?;
                file.write_all("[]".as_bytes())?;
            }
            None => {}
        }
        Ok(())
    }
}
