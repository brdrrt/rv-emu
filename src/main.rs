#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hides console window on Windows in release

use std::{fs, process::Command};

use egui_dock::{DockArea, NodeIndex, Style, Tree};
use egui_memory_editor::MemoryEditor;
use emu::{
    cpu::{CpuError, DecodeError},
    create_rv32,
    machine::Machine,
    memory::{
        constants::{MEMORY_SIZE, RAM_BASE},
        MemoryBus,
    },
};
use log::{debug, error};

use eframe::{
    egui::{self, Button, Ui},
    NativeOptions,
};

fn main() -> eframe::Result<()> {
    env_logger::init();
    let options = NativeOptions::default();
    eframe::run_native("rvemu", options, Box::new(|_cc| Box::<MyApp>::default()))
}

struct TabViewer<'a> {
    machine: &'a mut Machine,
    /// Wether the emulator has reached an instruction with opcode equal to zero
    has_reached_end: &'a mut bool,
    mem_editor: &'a mut MemoryEditor,
    code: &'a mut String,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = String;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab.as_str() {
            // TODO: Use enum instead of strings
            "Editor" => self.editor_pane(ui),
            "Registers" => self.registers_pane(ui),
            "Memory" => self.memory_pane(ui),
            _ => {
                ui.label(format!("Content of {tab}"));
            }
        };
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        (&*tab).into()
    }
}

impl TabViewer<'_> {
    fn editor_pane(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("Reset & Compile & Load in memory").clicked() {
                debug!("Resetting machine");
                debug!("Assembling code: {:?}", &self.code);
                let output = Command::new("rvasm")
                    .arg("-s")
                    .arg(&self.code)
                    .arg("-o")
                    .arg("out.bin")
                    .arg("-a")
                    .arg("RV32I")
                    .arg("-f")
                    .arg("flat")
                    .output();
                if let Err(output) = output {
                    error!("Error while assembling code: {output}");
                } else if let Ok(output) = output {
                    if !output.status.success() {
                        error!(
                            "Error while assembling code (from assembler):\nstderr: {:?}\nExit status: {:?}",
                            String::from_utf8(output.stderr)
                                .expect("Couldn't parse the assembler's stderr as UTF-8"),
                            output
                                .status
                                .code()
                                .expect("Couldn't get the assembler's exit status")
                        )
                    } else {
                        debug!("Successfully assembled code")
                    }
                }

                *self.machine =
                    create_rv32(fs::read("out.bin").expect("Couldn't read assembled file"));

                *self.has_reached_end = false;
            }

            if !*self.has_reached_end {
                let advance_button = ui.button("Step >>");
                let tillend_button = ui.button("Run until end");

                // TODO: Find better way, removing duplication for the buttons
                if advance_button.clicked() {
                    let mut memory_bus = MemoryBus::new(&mut self.machine.memory);
                    if let Err(error) = self.machine.cpu.advance(&mut memory_bus) {
                        // Reaching an instruction with opcode zero shouldn't be considered an error as it is actually expected here and it signals the end of the program
                        if let CpuError::Decode(DecodeError::OpcodeZero) = error {
                            *self.has_reached_end = true;
                        } else {
                            error!("Error while executing single instruction: {:?}", error);
                        }
                    }
                }

                if tillend_button.clicked() {
                    let mut memory_bus = MemoryBus::new(&mut self.machine.memory);
                    if let Err(error) = self.machine.cpu.reset(&mut memory_bus) {
                        // Reaching an instruction with opcode zero shouldn't be considered an error as it is actually expected here and it signals the end of the program
                        if let CpuError::Decode(DecodeError::OpcodeZero) = error {
                            *self.has_reached_end = true;
                        } else {
                            error!("Error while executing single instruction: {:?}", error);
                        }
                    }
                }
            } else {
                ui.add_enabled(false, Button::new("Step >>"));
                ui.add_enabled(false, Button::new("Run until end"));
            }
        });
        ui.add_sized(
            ui.available_size(),
            egui::TextEdit::multiline(self.code).code_editor(),
        );
    }

    fn registers_pane(&mut self, ui: &mut Ui) {
        egui::Grid::new("grid")
            .num_columns(3)
            .min_col_width(18.0)
            .striped(true)
            .show(ui, |ui| {
                ui.add_sized(ui.available_size(), |ui: &mut Ui| {
                    ui.label(format!("Register"))
                });
                ui.add_sized(ui.available_size(), |ui: &mut Ui| ui.label("Binary"));
                ui.add_sized(ui.available_size(), |ui: &mut Ui| {
                    ui.label(format!("Two's complement"))
                });
                ui.end_row();
                for (i, register) in self.machine.cpu.registers.iter().enumerate() {
                    ui.add_sized(ui.available_size(), |ui: &mut Ui| ui.label(format!("x{i}")));
                    ui.add_sized(ui.available_size(), |ui: &mut Ui| {
                        ui.label(register.to_string())
                    });
                    ui.add_sized(ui.available_size(), |ui: &mut Ui| {
                        let sign = register >> 31 & 0b1;
                        if sign == 1 {
                            let mut num = !*register;
                            num = num.wrapping_add(1);
                            ui.label(format!("-{}", num))
                        } else {
                            ui.label(register.to_string())
                        }
                    });
                    ui.end_row();
                }
            });
    }

    fn memory_pane(&mut self, ui: &mut Ui) {
        self.mem_editor.draw_editor_contents(
            ui,
            &mut self.machine.memory.contents, // TODO: Perhaps should use memory bus
            |mem, address| Some(mem[address - 0x80]), // TODO: Return none instead of some were applicable
            |_, _, _: u8| {}, // TODO: Think about making memory editable directly in the memory editor window
        );
    }
}

struct MyApp {
    tree: Tree<String>,
    machine: Machine,
    /// Wether the emulator has reached an instruction with opcode equal to zero
    has_reached_end: bool,
    mem_editor: MemoryEditor,
    code: String,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut tree = Tree::new(vec!["Editor".to_owned()]);

        // You can modify the tree before constructing the dock
        let [a, b] = tree.split_right(NodeIndex::root(), 0.7, vec!["Registers".to_owned()]);
        let [_, _] = tree.split_below(a, 0.6, vec!["Memory".to_owned()]);
        let [_, _] = tree.split_below(b, 0.5, vec!["Input/output".to_owned()]);

        Self {
            tree,
            code: "addi x2, x0, 20".to_owned(), // TODO: Remove hardcoded example code
            machine: Machine::new(vec![]),
            has_reached_end: false,
            // TODO: Maybe show other memory-mapped things too, not only physical memory
            mem_editor: MemoryEditor::new()
                .with_address_range("Physical memory", RAM_BASE..MEMORY_SIZE)
                .with_window_title("Memory editor"),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        DockArea::new(&mut self.tree)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show_close_buttons(false)
            .show(
                ctx,
                &mut TabViewer {
                    machine: &mut self.machine,
                    code: &mut self.code,
                    mem_editor: &mut self.mem_editor,
                    has_reached_end: &mut self.has_reached_end,
                },
            );
    }
}
