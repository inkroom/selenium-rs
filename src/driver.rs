use std::{
    collections::HashMap,
    fmt::Display,
    io::{BufRead, BufReader, Read},
    process::{Child, Command, Stdio},
    rc::Rc,
    thread::sleep,
    time::Duration,
};

use serde::{Deserialize, Serialize};

use crate::{
    actions::Action,
    element::Element,
    http::{Capability, Http},
    option::{Browser, BrowserOption},
    SError, SResult,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Session {
    pub(crate) session_id: String,
}

struct DriverProcess {
    driver: Child,
    monitor: Child,
}

impl Drop for DriverProcess {
    fn drop(&mut self) {
        let _ = self.monitor.kill();
        let _ = self.driver.kill();
        // 避免产生僵尸进程
        let _ = self.monitor.wait();
        let _ = self.driver.wait();
    }
}

impl DriverProcess {
    fn get_available_port() -> u16 {
        std::net::TcpListener::bind("0.0.0.0:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    }

    fn start_driver(
        exec: &str,
        env: &HashMap<String, String>,
        browser: Browser,
    ) -> SResult<(Child, u16)> {
        let port = Self::get_available_port();
        let mut s = Command::new(exec);
        match browser {
            Browser::Firefox => s.arg("--port").arg(port.to_string()),
            Browser::Chrome => s.arg(format!("--port={port}")),
            Browser::Safari => s.arg("--port").arg(port.to_string()),
            Browser::Edge => s.arg(format!("--port={port}")),
        };
        let mut s = s
            .envs(env)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            // .stdout(Stdio::from(std::fs::OpenOptions::new().create(true).append(true).open("1.log")?))
            // .stderr(Stdio::from(std::fs::OpenOptions::new().create(true).append(true).open("1e.log")?))
            .spawn()?;

        // 测试启动是否结束
        for _ in 0..3 {
            if let Err(e) = minreq::get(format!("http://127.0.0.1:{port}")).send() {
                if e.to_string().contains("refuse") {
                    // 连接失败，查看是否退出
                    match s.try_wait() {
                        Ok(Some(e)) => {
                            // 已经退出
                            return Err(SError::Driver(format!(
                                "start driver fail, exit code : {}",
                                e.code().unwrap_or(-1)
                            )));
                        }
                        Ok(None) => {
                            // 没有退出，就延迟等待
                            sleep(Duration::from_millis(500));
                        }
                        Err(e) => return Err(SError::Driver(e.to_string())),
                    }
                }
            };
        }
        Ok((s, port))
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn start_monitor(driver_pid: u32) -> SResult<Child> {
        let v = Command::new("/bin/sh")
            .arg("-c")
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .stdin(Stdio::null())
            .arg(format!(
                "while kill -0 {} 2>/dev/null; do sleep 0.5; done; kill {}", // do sleep 后跟着的就是定期检查的间隔时间
                std::process::id(),
                driver_pid
            ))
            .spawn()
            .map_err(|e| SError::Driver(e.to_string()))?;
        Ok(v)
    }

    #[cfg(target_os = "windows")]
    fn start_monitor(driver_pid: u32) -> SResult<Child> {
        let v = Command::new("powershell.exe")
            .arg("-Command")
            .arg(format!(
                "while ((Get-Process -Id {} -ErrorAction SilentlyContinue) -ne $null) \
            {{ Start-Sleep -Milliseconds 500 }}; Stop-Process -Id {} -Force",
                std::process::id(),
                driver_pid
            ))
            .spawn()
            .map_err(|e| SError::Driver(e.to_string()))?;
        Ok(v)
    }

    pub(crate) fn get_err(&mut self) -> Result<String, SError> {
        if let Some(v) = &mut self.driver.stderr {
            let mut r = timeout_readwrite::TimeoutReader::new(v, Duration::from_secs(2));
            // let mut r = BufReader::new(v);

            let mut res = String::new();
            return match r.read_to_string(&mut res) {
                Ok(v) => Ok(res),
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(res),
                Err(ref e) => Err(SError::Driver(e.to_string())),
            };
        }

        Ok(String::new())
    }

    pub(crate) fn new(
        driver: &str,
        env: &HashMap<String, String>,
        browser: Browser,
    ) -> SResult<(Self, u16)> {
        let mut c = Self::start_driver(driver, env, browser)?;
        let m = Self::start_monitor(c.0.id())?;
        if let Ok(Some(exit)) = c.0.try_wait() {
            if let Some(v) = c.0.stderr {
                let mut r = timeout_readwrite::TimeoutReader::new(v, Duration::from_secs(2));

                let mut res = String::new();
                return match r.read_to_string(&mut res) {
                    Ok(v) => Err(SError::Driver(format!("start driver fail,reason: {res}"))),
                    Err(e) => Err(SError::Driver(format!(
                        "start driver fail,exit {}",
                        exit.code().unwrap_or(-1)
                    ))),
                };
            }
            return Err(SError::Driver(format!(
                "start driver fail,exit {}",
                exit.code().unwrap_or(-1)
            )));
        }
        Ok((
            Self {
                driver: c.0,
                monitor: m,
            },
            c.1,
        ))
    }
}

pub struct Driver {
    pub(crate) session: Rc<Session>,
    pub(crate) http: Rc<Http>,
    process: Option<DriverProcess>,
    browser: Browser,
}
#[cfg(debug_assertions)]
impl Drop for Driver {
    fn drop(&mut self) {
        let _ = self.quit();
    }
}

pub enum NewWindowType {
    Tab,
    Window,
}

pub enum SwitchToFrame {
    Null,
    Number(usize),
    Element(String),
}
/// 单位都是毫秒
pub enum TimeoutType {
    Script(u32),
    PageLoad(u32),
    Implicit(u32),
}

pub enum By<'a> {
    Css(&'a str),
    LinkText(&'a str),
    ParitialLinkText(&'a str),
    TagName(&'a str),
    XPath(&'a str),
    Class(&'a str),
    Id(&'a str),
}

#[derive(Deserialize, Clone, Serialize, Debug)]
pub struct Rect {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f32>,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Rect {
            x: Some(x),
            y: Some(y),
            width: Some(width),
            height: Some(height),
        }
    }

    pub fn size(width: f32, height: f32) -> Self {
        Rect {
            x: Some(0.0),
            y: Some(0.0),
            width: Some(width),
            height: Some(height),
        }
    }
}

impl Display for NewWindowType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tab => f.write_str("tab"),
            Self::Window => f.write_str("window"),
        }
    }
}
impl Driver {
    pub fn new(option: impl BrowserOption) -> SResult<Self> {
        // 连接远程
        if let Some(url) = option.url() {
            #[cfg(not(feature = "https"))]
            if url.contains("https://") {
                panic!("enable https features to use https protocol");
            }

            let http = Http::new(url, option.timeout());
            let b = option.browser();
            // 开启session
            let cap = Capability {
                browser_name: Some(format!("{}", option.browser())),
                platform_name: None,
                always_match: Some(option),
            };
            let session = http.new_session(cap)?;
            return Ok(Driver {
                session: Rc::new(session),
                http: Rc::new(http),
                process: None,
                browser: b,
            });
        } else if let Some(driver) = option.driver() {
            let b = option.browser();
            // 启用driver进程
            let (mut s, port) = DriverProcess::new(driver, option.env(), option.browser())?;
            let http = Http::new(
                format!("http://127.0.0.1:{port}").as_str(),
                option.timeout(),
            );
            // 开启session
            let cap = Capability {
                browser_name: None,
                platform_name: None,
                always_match: Some(option),
            };

            match http.new_session(cap) {
                Ok(session) => {
                    return Ok(Driver {
                        session: Rc::new(session),
                        http: Rc::new(http),
                        process: Some(s),
                        browser: b,
                    });
                }
                Err(e) => {
                    if let SError::Http(status, err) = &e {
                        if status == &500 {
                            if err.contains("Process unexpectedly closed with status 1") {
                                // firefox driver启动失败
                                if let Ok(v) = s.get_err() {
                                    return Err(SError::Driver(format!("Driver error msg: {v}")));
                                } else {
                                    return Err(SError::Driver(format!("{}", err)));
                                }
                            }
                        }
                    }

                    return Err(e);
                }
            }
        }
        unimplemented!()
    }

    pub fn browser(&self) -> Browser {
        self.browser.clone()
    }

    pub fn quit(&self) -> SResult<()> {
        self.http.delete_session(&self.session.session_id)
    }

    pub fn get(&self, url: &str) -> SResult<()> {
        self.http.navigate(&self.session.session_id, url)
    }

    pub fn get_current_url(&self) -> SResult<String> {
        self.http.get_current_url(&self.session.session_id)
    }

    pub fn forward(&self) -> SResult<()> {
        self.http.forward(&self.session.session_id)
    }

    pub fn refresh(&self) -> SResult<()> {
        self.http.refresh(&self.session.session_id)
    }

    pub fn back(&self) -> SResult<()> {
        self.http.back(&self.session.session_id)
    }

    pub fn get_title(&self) -> SResult<String> {
        self.http.get_title(&self.session.session_id)
    }

    pub fn set_timeouts(&self, timeout: TimeoutType) -> SResult<()> {
        self.http.set_timeouts(&self.session.session_id, timeout)
    }

    pub fn get_timeouts(&self) -> SResult<Vec<TimeoutType>> {
        self.http.get_timouts(&self.session.session_id)
    }
}
/// contenxts
impl Driver {
    pub fn get_window_handle(&self) -> SResult<String> {
        self.http.get_window_handle(&self.session.session_id)
    }

    pub fn get_window_handles(&self) -> SResult<Vec<String>> {
        self.http.get_window_handles(&self.session.session_id)
    }

    pub fn close_window(&self) -> SResult<Vec<String>> {
        self.http.close_window(&self.session.session_id)
    }

    pub fn new_window(&self, window_type: NewWindowType) -> SResult<String> {
        self.http
            .new_window(&self.session.session_id, format!("{window_type}").as_str())
    }
    pub fn switch_to_window(&self, handle: &str) -> SResult<()> {
        self.http.switch_to_window(&self.session.session_id, handle)
    }

    pub fn switch_to_frame(&self, id: SwitchToFrame) -> SResult<()> {
        self.http.switch_to_frame(&self.session.session_id, id)
    }
    ///
    /// https://w3c.github.io/webdriver/#switch-to-parent-frame
    pub fn switch_to_parent_frame(&self) -> SResult<()> {
        self.http.switch_to_parent_frame(&self.session.session_id)
    }

    pub fn get_window_rect(&self) -> SResult<Rect> {
        self.http.get_window_rect(&self.session.session_id)
    }

    pub fn set_window_rect(&self, rect: Rect) -> SResult<Rect> {
        self.http.set_window_rect(&self.session.session_id, rect)
    }

    pub fn maximize_window(&self) -> SResult<Rect> {
        self.http.maximize_window(&self.session.session_id)
    }

    pub fn minimize_window(&self) -> SResult<Rect> {
        self.http.minimize_window(&self.session.session_id)
    }

    pub fn fullscreen_window(&self) -> SResult<Rect> {
        self.http.fullscreen_window(&self.session.session_id)
    }
}

/// element
impl Driver {
    pub fn find_element(&self, by: By<'_>) -> SResult<Element> {
        let v = self.http.find_element(&self.session.session_id, by)?;
        Ok(Element {
            http: Rc::clone(&self.http),
            session: Rc::clone(&self.session),
            identify: v.0,
            id: v.1,
        })
    }

    pub fn find_elements(&self, by: By<'_>) -> SResult<Vec<Element>> {
        let v = self.http.find_elements(&self.session.session_id, by)?;
        Ok(v.iter()
            .map(|f| Element {
                http: Rc::clone(&self.http),
                session: Rc::clone(&self.session),
                identify: f.0.clone(),
                id: f.1.clone(),
            })
            .collect())
    }

    pub fn get_active_element(&self) -> SResult<Element> {
        let v = self.http.get_active_element(&self.session.session_id)?;
        Ok(Element {
            http: Rc::clone(&self.http),
            session: Rc::clone(&self.session),
            identify: v.0,
            id: v.1,
        })
    }
}

// document
impl Driver {
    pub fn get_page_source(&self) -> SResult<String> {
        self.http.get_page_source(&self.session.session_id)
    }
    ///
    /// 由于脚本执行返回的数据类型相当复杂，而且协议里并没有规定告知返回的数据类型，所以区分部分情况几乎不可能
    ///
    /// 建议执行的脚本只返回基础数据类型
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use selenium::option::FirefoxBuilder;
    /// use selenium::driver::Driver;
    /// let driver = Driver::new(FirefoxBuilder::new().build()).unwrap();
    /// let _:() = driver.execute_script("location.reload()", &[]).unwrap();
    /// ```
    ///
    pub fn execute_script<T: serde::de::DeserializeOwned>(
        &self,
        script: &str,
        args: &[&str],
    ) -> SResult<T> {
        self.http
            .execute_script::<T>(&self.session.session_id, script, args)
    }

    pub fn dismiss_alert(&self) -> SResult<()> {
        self.http.dismiss_alert(&self.session.session_id)
    }

    pub fn accept_alert(&self) -> SResult<()> {
        self.http.accept_alert(&self.session.session_id)
    }

    pub fn get_alert_text(&self) -> SResult<String> {
        self.http.get_alert_text(&self.session.session_id)
    }
    ///
    /// 设置prompt 输入的文本
    ///
    /// 注意，设置文本后需要调用 `accept_alert` 点击确认按钮，否则alert依然存在
    ///
    pub fn send_alert_text(&self, text: &str) -> SResult<()> {
        self.http.send_alert_text(&self.session.session_id, text)
    }

    pub fn take_screenshot(&self) -> SResult<Vec<u8>> {
        self.http.take_screenshot(&self.session.session_id)
    }
}

impl Driver {
    pub fn actions(&self) -> Action {
        Action::new(Rc::clone(&self.http), Rc::clone(&self.session))
    }
}
