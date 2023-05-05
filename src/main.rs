mod API;

use std::env;
use dotenv::dotenv;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, BorderType, Clear, List, ListItem, Paragraph, Wrap};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent};
use crossterm::{event, execute};

use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, size};
use tui::backend::Backend;
use tui::{Frame, Terminal};
use tui::layout::{Alignment, Constraint, Corner, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};

//import thread and duration
use std::time::Duration;
use std::time::Instant;
use crossterm::event::{poll};
use serde::{Serialize, Deserialize};
use xml::reader::{EventReader, XmlEvent};


use crate::API::{poweron_vm, reboot_vm, shutdown_vm};


const APP_KEYS_DESC: &str = r#"
S:           Search Mode
H:           Host Mode
F:           Select File
Up:          Select Previous Vm
Down:        Select Next Vm
Intro:       Select Host/VM
O:           PowerOff VMs
P:           PowerOn Vms
R:           Reboot Vms
Esc:         Exit
"#;


//Enum for the different objects

enum InputMode {
    Normal,
    Search,
    ListVM,
    File,
}

impl Clone for InputMode {
    fn clone(&self) -> InputMode {
        match self {
            InputMode::Normal => InputMode::Normal,
            InputMode::Search => InputMode::Search,
            InputMode::ListVM => InputMode::ListVM,
            InputMode::File => InputMode::File,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct PowerTUI {
    vms: Vec<String>,
}

struct Vm {
    name: String,
    state: String,
    vm_id: String,
    selected: bool,
    error: bool,
    error_msg: String,
    delete: bool,
}

impl Clone for Vm {
    fn clone(&self) -> Vm {
        Vm {
            name: self.name.clone(),
            state: self.state.clone(),
            vm_id: self.vm_id.clone(),
            selected: self.selected.clone(),
            error: self.error.clone(),
            error_msg: self.error_msg.clone(),
            delete: self.delete.clone(),
        }
    }
}

struct Host {
    name: String,
    state: String,
    ip: String,
    vms: Vec<Vm>,
    cpy_vms: Vec<Vm>,
}

impl Clone for Host {
    fn clone(&self) -> Host {
        Host {
            name: self.name.clone(),
            state: self.state.clone(),
            ip: self.ip.clone(),
            vms: self.vms.clone(),
            cpy_vms: self.cpy_vms.clone(),
        }
    }
}

struct State {
    mode: InputMode,
    hosts: Vec<Host>,
    host_cursor: Option<usize>,
    vm_cursor: Option<usize>,
    search_string : String,
    file_path: String,
    input: KeyEvent,
}

impl Clone for State {
    fn clone(&self) -> State {
        State {
            mode: self.mode.clone(),
            hosts: self.hosts.clone(),
            host_cursor: self.host_cursor.clone(),
            vm_cursor: self.vm_cursor.clone(),
            search_string: self.search_string.clone(),
            input: self.input.clone(),
            file_path: self.file_path.clone(),
        }
    }
}

impl Vm {
    fn new(name: &str, state: &str,vm : &str) -> Vm {
        Vm {
            name: name.to_string(),
            state: state.to_string(),
            selected: false,
            vm_id: vm.to_string(),
            error: false,
            error_msg: "".to_string(),
            delete: false,
        }
    }
}

impl Host {
    fn new(name: &str, state: &str, ip: &str) -> Host {
        Host {
            name: name.to_string(),
            state: state.to_string(),
            ip: ip.to_string(),
            vms: Vec::new(),
            cpy_vms: Vec::new(),
        }
    }

    fn add_vm(&mut self, vm: Vm) {
        self.vms.push(vm);
    }
    fn add_cpy_vm(&mut self, vm: Vm) {
        self.cpy_vms.push(vm);
    }
}

impl State {
    fn new() -> State {
        State{
            mode: InputMode::Normal,
            hosts: Vec::new(),
            host_cursor: None,
            vm_cursor: None,
            search_string: "".to_string(),
            file_path: "C:\\Users\\a2780\\Desktop\\vms.yaml".to_string(),
            //create input
            input: KeyEvent::from(KeyCode::Null),
        }
    }

}

#[derive(Serialize, Deserialize)]
struct Credentials {
    ip: String,
    user: String,
    password: String,
    datacenter: String,
}

fn search(state : &mut State) {

    state.hosts.iter_mut().for_each(|host| {
        host.cpy_vms.iter_mut().for_each(|vm| {
            if !(vm.name.to_lowercase().contains(&state.search_string.to_lowercase())) {
                vm.delete = true;
            }else{
                vm.delete = false;
            }
        });
    });



}

fn delete(state : &mut State) {
    state.hosts.iter_mut().for_each(|host| {
        //create a temporal copy of the vms
        let mut vms = host.cpy_vms.clone();
        //delete the vms that are marked as delete
        vms.retain(|vm| !vm.delete);
        //replace the vms with the new vms
        if vms.len() > 0 {
            host.vms = vms;
        }
    });
}


fn load_crendetials() -> (String, String, String, String) {
    //open the file credentials.xml who is in the src folder
    let file = File::open("src/credentials.json").unwrap();
    let reader = BufReader::new(file);
    let credentials: Credentials = serde_json::from_reader(reader).unwrap();

    //return the credentials

    return (credentials.ip, credentials.user, credentials.password, credentials.datacenter);

}


fn main() -> Result<(), Box<dyn Error>> {
    let (hostname, username, password, datacenter) = load_crendetials();

    let api = API::new_api(hostname.clone());



    //run API main func
    let credentials = API::authenticate(api.clone()
                                                    ,username
                                                    ,password
                                                     , datacenter
    );


    enable_raw_mode()?;
    execute!(std::io::stdout(), EnableMouseCapture, EnterAlternateScreen)?;
    let mut state = State::new();

    let hosts = API::get_all_hosts(api.clone(),credentials.clone());

    hosts.iter().for_each(|host| {
        state.hosts.push(Host::new(&host.name, &host.power_state, &host.host));

        let vms = API::get_vms_from_host(api.clone(),credentials.clone(), host.clone().name);
        vms.iter().for_each(|vm| {
            state.hosts.last_mut().unwrap().add_vm(Vm::new(&vm.name, &vm.power_state,&vm.vm));
            state.hosts.last_mut().unwrap().add_cpy_vm(Vm::new(&vm.name, &vm.power_state,&vm.vm));

        });

    });


    state.host_cursor = Some(0);
    


    let backend = tui::backend::CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let result = run_app(&mut terminal,credentials,&mut state, Duration::from_millis(100),hostname);

    disable_raw_mode()?;

    execute!(terminal.backend_mut(), DisableMouseCapture,LeaveAlternateScreen)?;

    if let Err(e) = result {
        println!("{}", e.to_string());
    }

    Ok(())


}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    credentials: String,
    state: &mut State,
    tick_rate: Duration,
    hostname: String,
)
    -> Result<(), std::io::Error> {

    let mut last_tick = Instant::now();
    let api = API::new_api(hostname);

    loop {
        terminal.draw(|f| ui(f, state))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match state.mode {
                    InputMode::Normal => {
                        match key.code {
                            KeyCode::Esc => {
                                disable_raw_mode()?;
                                return Ok(());
                            }
                            KeyCode::Down => {
                                    if state.host_cursor.unwrap_or(0) < (state.hosts.len() - 1) {
                                        state.host_cursor = Some(state.host_cursor.unwrap_or(0).saturating_add(1));
                                    }
                            }
                            KeyCode::Up=> {
                                    if state.host_cursor.unwrap_or(0) > 0 {
                                        state.host_cursor = Some(state.host_cursor.unwrap_or(0).saturating_sub(1));
                                    }
                            }
                            KeyCode::Enter => {
                                state.mode = InputMode::ListVM;
                                state.vm_cursor = Some(0);
                            }

                            KeyCode::Char('s') => {
                                state.mode = InputMode::Search;
                            }

                            KeyCode::Char('f') => {
                                state.mode = InputMode::File;
                            }
                            _ => {
                            }
                        }
                    }
                    InputMode::ListVM => {
                        match key.code {

                            KeyCode::Char('h') => {
                                state.mode = InputMode::Normal;
                                state.vm_cursor = None;
                            }
                            KeyCode::Up => {

                                    if state.vm_cursor.unwrap_or(0) > 0 {
                                        state.vm_cursor = Some(state.vm_cursor.unwrap_or(0).saturating_sub(1));
                                    }
                            }
                            KeyCode::Down => {
                                if state.vm_cursor.unwrap_or(0) < (state.hosts[state.host_cursor.unwrap_or(0)].vms.len() - 1) {
                                    state.vm_cursor = Some(state.vm_cursor.unwrap_or(0).saturating_add(1));
                                }
                            }
                            KeyCode::Char(' ') => {
                                    let mut vm = &mut state.hosts[state.host_cursor.unwrap_or(0)].vms[state.vm_cursor.unwrap_or(0)];
                                    //edit selected
                                    vm.selected = !vm.selected;
                            }
                            KeyCode::Char('p') => {
                             //   if key_pressed {
                                    let mut vms = state.hosts[state.host_cursor.unwrap_or(0)]
                                        .vms
                                        .iter_mut()
                                        .filter(|x| x.selected)
                                        .collect::<Vec<&mut Vm>>();

                                    vms.iter_mut().for_each(|x| {
                                        let statuscode = poweron_vm(api.clone(), credentials.clone(), x.vm_id.clone());

                                        if statuscode == 200 {
                                            //unselect vm and set state to POWERED_ON
                                            x.selected = false;
                                            x.state = "POWERED_ON".to_string();
                                            x.error = false;

                                        } else {
                                            x.error = true;
                                            x.error_msg = ("Error While Powering on vm".to_string())

                                        }
                                    });
                            }
                            KeyCode::Char('o') => {

                                    let mut vms = state.hosts[state.host_cursor.unwrap_or(0)]
                                        .vms
                                        .iter_mut()
                                        .filter(|x| x.selected)
                                        .collect::<Vec<&mut Vm>>();
                                    vms.iter_mut().for_each(|x| {
                                        let statuscode = shutdown_vm(api.clone(), credentials.clone(), x.vm_id.clone());

                                        if statuscode == 200 {
                                            //unselect vm and set state to POWERED_ON
                                            x.selected = false;
                                            x.state = "POWERED_OFF".to_string();
                                            x.error = false;
                                        } else {
                                            x.error = true;
                                            x.error_msg = ("Error While Powering on vm".to_string())
                                        }
                                    });
                            }
                            KeyCode::Char('r') => {

                                    let mut vms = state.hosts[state.host_cursor.unwrap_or(0)]
                                        .vms
                                        .iter_mut()
                                        .filter(|x| x.selected)
                                        .collect::<Vec<&mut Vm>>();
                                    vms.iter_mut().for_each(|x| {
                                        let statuscode = reboot_vm(api.clone(), credentials.clone(), x.vm_id.clone());

                                        if statuscode == 200 {
                                            //unselect vm and set state to POWERED_ON
                                            x.selected = false;
                                            x.state = "POWERED_ON".to_string();
                                            x.error = false;
                                            x.error_msg = ("Error While Powering on vm".to_string())
                                        } else {
                                            x.error = true;
                                            x.error_msg = ("Error While Powering on vm".to_string())

                                        }
                                    });
                            }
                            _ => {
                            }
                        }
                    }
                    InputMode::Search => {
                        match key.code {
                             KeyCode::Esc => {
                                    state.mode = InputMode::Normal;
                                    state.vm_cursor = None;
                            }
                            KeyCode::Char(c) => {
                                    state.search_string.push(c);
                                    search(state);
                                    delete(state);
                            }

                            KeyCode::Backspace => {
                                    state.search_string.pop();
                                    search(state);
                                    delete(state);
                            }
                            _ => {

                            }
                        }
                    }
                    InputMode::File =>{
                        match key.code {
                            KeyCode::Char(c) => {
                                    state.file_path.push(c);
                            }

                            KeyCode::Enter => {
                                    state.mode = InputMode::Normal;
                                    select_by_file(state);
                                    state.file_path = "".to_string();
                            }

                            KeyCode::Backspace => {
                                    state.file_path.pop();
                            }

                            _ => {

                            }
                        }
                    }
                }
            } else {

            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, state: &mut State) {
    f.size();

    let parent_layout = Layout::default()
        .direction(tui::layout::Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(30),
                Constraint::Percentage(70)
            ].as_ref()
        )
        .split(f.size());

    let new_section_block = Block::default()
        .title("Info")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    f.render_widget(new_section_block, parent_layout[0]);
    show_info(f,state, parent_layout[0]);


    let new_list_section = Block::default()
        .title("Datacenter")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    f.render_widget(new_list_section, parent_layout[1]);
    list_section(f,state, parent_layout[1]);

    file_popup(f, state);


}

fn list_section<B: Backend>(f: &mut Frame<B>, state: &mut State, rect: Rect) {
    let items: Vec<ListItem> = state
        .hosts
        .iter()
        .enumerate()
        .map(|(index,host)| {
            //create a host_item empty

            let mut host_item = ListItem::new(format!("\n• {}", host.name))
                .style(Style::default().fg(Color::Red));
            let selected_host = state.host_cursor == Some(index);
            if selected_host {
                host_item = ListItem::new(format!("\n• {}", host.name))
                    .style(Style::default().fg(Color::Green));
            } else {
                host_item = ListItem::new(format!("\n• {}", host.name))
                    .style(Style::default().fg(Color::Gray));
            }

            let vm_items: Vec<ListItem> = host
                .vms
                .iter()
                .enumerate()
                .map(|(index,vm)| {
                    return if selected_host && state.vm_cursor == Some(index) {
                        if vm.selected {
                            ListItem::new(format!(" [x]   {} - {}", vm.name, vm.state))
                                .style(Style::default().fg(Color::Green))
                        } else {
                            ListItem::new(format!(" [ ]   {} - {}", vm.name, vm.state))
                                .style(Style::default().fg(Color::Green))
                        }
                    } else if selected_host {
                        if vm.selected {
                            //create a list item with the vm.name in color white and the vm.sate in color green

                            ListItem::new(format!(" [x]   {} - {}", vm.name, vm.state))
                                .style(Style::default().fg(Color::White))
                        } else {
                            ListItem::new(format!(" [ ]   {} - {}", vm.name, vm.state))
                                .style(Style::default().fg(Color::White))
                        }
                    }else{
                        if vm.selected {
                            ListItem::new(format!(" [x]   {} - {}", vm.name, vm.state))
                                .style(Style::default().fg(Color::Gray))
                        } else {
                            ListItem::new(format!(" [ ]   {} - {}", vm.name, vm.state))
                                .style(Style::default().fg(Color::Gray))
                        }
                    }
                })
                .collect();

            if vm_items.len() > 0 {
                vec![host_item, ListItem::new("\n")]
                .into_iter()
                .chain(vm_items.into_iter())
                .collect()
            } else {
                vec![host_item]
            }

        })
        .flatten()
        .collect();


    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Datacenter"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ")
        .start_corner(Corner::TopLeft);



    f.render_widget(list, rect);
}

fn display_host_info<B: Backend>(f : &mut Frame<B>, state: &mut State, new_section_chunk: Vec<Rect>){
    let text = vec![
        Spans::from(vec![Span::styled(format!("{}", state.hosts[state.host_cursor.unwrap_or(0)].name),
                                      Style::default().fg(Color::White),
        )]),
    ];
    let text2 = vec![
        if state.hosts[state.host_cursor.unwrap_or(0)].state == "POWERED_ON" {
            Spans::from(vec![Span::styled(format!("{}", state.hosts[state.host_cursor.unwrap_or(0)].state),
                                          Style::default().fg(Color::LightGreen),
            )])
        } else {
            Spans::from(vec![Span::styled(format!("{}", state.hosts[state.host_cursor.unwrap_or(0)].state),
                                          Style::default().fg(Color::LightRed),
            )])
        },
    ];
    let text3 = vec![Spans::from(vec![Span::styled(format!("{}", state.hosts[state.host_cursor.unwrap_or(0)].vms.len()),
                                                   Style::default().fg(Color::White),
    )]),];

    let text4 = vec![Spans::from(vec![Span::styled(format!("{}", state.hosts[state.host_cursor.unwrap_or(0)].vms.iter().filter(|vm| vm.state == "POWERED_ON").count()),
                                                   Style::default().fg(Color::White),
    )]),];
    //clear the screen
    f.render_widget(Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("Host Name")), new_section_chunk[1]);
    f.render_widget(Paragraph::new(text2).block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("Host State")), new_section_chunk[2]);
    f.render_widget(Paragraph::new(text3).block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("VM number")), new_section_chunk[3]);
    f.render_widget(Paragraph::new(text4).block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("Powered ON VMs")), new_section_chunk[4]);
}

fn show_info<B: Backend>(f: &mut Frame<B>, state: &mut State, rect: Rect) {
    let new_section_chunk = Layout::default()
        .margin(2)
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(15),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ].as_ref()
        )
        .split(rect);

    let desc = Paragraph::new(APP_KEYS_DESC);
    f.render_widget(desc, new_section_chunk[0]);

    match state.mode {
        InputMode::Normal => {
            let text = vec![
                Spans::from(vec![Span::styled(format!("{}", state.hosts[state.host_cursor.unwrap_or(0)].name),
                                              Style::default().fg(Color::White),
                )]),
            ];
            let text2 = vec![
                if (state.hosts[state.host_cursor.unwrap_or(0)].state == "POWERED_ON") {
                    Spans::from(vec![Span::styled(format!("{}", state.hosts[state.host_cursor.unwrap_or(0)].state),
                                                  Style::default().fg(Color::LightGreen),
                    )])
                } else {
                    Spans::from(vec![Span::styled(format!("{}", state.hosts[state.host_cursor.unwrap_or(0)].state),
                                                  Style::default().fg(Color::LightRed),
                    )])
                },
            ];
            let text3 = vec![Spans::from(vec![Span::styled(format!("{}", state.hosts[state.host_cursor.unwrap_or(0)].vms.len()),
                                                           Style::default().fg(Color::White),
            )]),];

            let text4 = vec![Spans::from(vec![Span::styled(format!("{}", state.hosts[state.host_cursor.unwrap_or(0)].vms.iter().filter(|vm| vm.state == "POWERED_ON").count()),
                                                           Style::default().fg(Color::White),
            )]),];
            //clear the screen
            f.render_widget(Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("Host Name")), new_section_chunk[1]);
            f.render_widget(Paragraph::new(text2).block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("Host State")), new_section_chunk[2]);
            f.render_widget(Paragraph::new(text3).block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("VM number")), new_section_chunk[3]);
            f.render_widget(Paragraph::new(text4).block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("Powered ON VMs")), new_section_chunk[4]);
            let username_input = Paragraph::new(state.search_string.to_owned())
                .block(Block::default().title("Search VM by Name").borders(Borders::ALL).border_type(BorderType::Rounded))
                .style(Style::default().fg(Color::Gray),
                );
            f.render_widget(username_input, new_section_chunk[6]);
        }
        InputMode::ListVM => {
            let text = vec![

                //span with the vm name
                Spans::from(vec![Span::styled(format!("{}", state.hosts[state.host_cursor.unwrap_or(0)].vms[state.vm_cursor.unwrap_or(0)].name),
                                              Style::default().fg(Color::White),
                )]),];


            let text2 = vec![
                if state.hosts[state.host_cursor.unwrap_or(0)].vms[state.vm_cursor.unwrap_or(0)].state == "POWERED_ON" {
                    Spans::from(vec![Span::styled(format!("{}", state.hosts[state.host_cursor.unwrap_or(0)].vms[state.vm_cursor.unwrap_or(0)].state),
                                                  Style::default().fg(Color::LightGreen), )])
                } else {
                    Spans::from(vec![Span::styled(format!("{}", state.hosts[state.host_cursor.unwrap_or(0)].vms[state.vm_cursor.unwrap_or(0)].state),
                                                  Style::default().fg(Color::LightRed), )])
                },
            ];

             let text3 = vec![
                 if state.hosts[state.host_cursor.unwrap_or(0)].vms[state.vm_cursor.unwrap_or(0)].error == true {
                 Spans::from(vec![Span::styled(format!("{}", state.hosts[state.host_cursor.unwrap_or(0)].vms[state.vm_cursor.unwrap_or(0)].error_msg),
                                               Style::default().fg(Color::LightRed),
                 )])
             } else {
                 Spans::from(vec![Span::styled(format!("\n\nNo errors"),
                                               Style::default().fg(Color::LightGreen),
                 )])
             }
             ];
            //render the pra
            f.render_widget(Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("VM Name")), new_section_chunk[1]);

            f.render_widget(Paragraph::new(text2).block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("VM State")), new_section_chunk[2]);
            f.render_widget(Paragraph::new(text3).block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("Error")), new_section_chunk[3]);


        }
        InputMode::Search => {



            let username_input = Paragraph::new(state.search_string.to_owned())
                .block(Block::default().title("Search VM by Name").borders(Borders::ALL).border_type(BorderType::Rounded))
                .style(Style::default().fg(Color::Yellow),
                );
            f.render_widget(username_input, new_section_chunk[6]);

            display_host_info(f, state, new_section_chunk);
        }
        _ => {}
    }

}

fn file_popup<B: Backend>(f: &mut Frame<B>, state: &mut State) {
    if let InputMode::File = state.mode {
        let block = Block::default()
            .title("Find File")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        let area = centered_rect(60, 25, f.size());
        f.render_widget(Clear, area); //this clears out the background
        f.render_widget(block, area);

        let chunk = Layout::default()
            .margin(2)
            .constraints(
                [
                    Constraint::Length(2),
                    Constraint::Length(3),
                    Constraint::Length(2),
                ].as_ref()
            )
            .split(area);

        let text = Paragraph::new("Enter the path to the file with the vms you like to select (File should be a YAML)")
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        f.render_widget(text, chunk[0]);

        let path_input = Paragraph::new(state.file_path.to_owned())
            .block(Block::default().title("Search VM by Name").borders(Borders::ALL).border_type(BorderType::Rounded))
            .style(Style::default().fg(Color::White)    ,
            );
        f.render_widget(path_input, chunk[1]);
    }
}

fn select_by_file(state : &mut State){

    state.search_string = state.file_path.to_owned();

    //read the file
    let file = File::open(state.file_path.to_owned()).unwrap();
    let reader = BufReader::new(file);
    let parser = EventReader::new(reader);

    let mut names = Vec::new();
    let mut current_name: Option<String> = None;

    for event in parser {
        match event {
            Ok(XmlEvent::StartElement { name, .. }) => {
                if name.local_name == "name" {
                    current_name = Some(String::new());
                }
            }
            Ok(XmlEvent::EndElement { name }) => {
                if name.local_name == "name" {
                    if let Some(name) = current_name.take() {
                        names.push(name);
                    }
                }
            }
            Ok(XmlEvent::Characters(chars)) => {
                if let Some(ref mut name) = current_name {
                    name.push_str(&chars);
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    state.hosts.iter_mut().for_each(|host| {
        host.vms.iter_mut().for_each(|vm| {
            if names.contains(&vm.name) {
                vm.selected = true;
            }
        })
    });




}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
                .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
                .as_ref(),
        )
        .split(popup_layout[1])[1]
}




