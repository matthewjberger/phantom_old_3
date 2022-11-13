use anyhow::Result;
use log;
use phantom::app::Resources;
use std::path::PathBuf;

pub trait Command {
    fn is_undoable(&self) -> bool;
    fn execute(&mut self, resources: &mut Resources) -> Result<()>;
    fn undo(&mut self, resources: &mut Resources) -> Result<()>;
}

#[derive(Default)]
pub struct CommandList {
    pending_commands: Vec<Box<dyn Command>>,
    undo_commands: Vec<Box<dyn Command>>,
    redo_commands: Vec<Box<dyn Command>>,
}

impl CommandList {
    pub fn has_undo_commands(&self) -> bool {
        !self.undo_commands.is_empty()
    }

    pub fn has_redo_commands(&self) -> bool {
        !self.redo_commands.is_empty()
    }

    pub fn queue_command(&mut self, command: Box<dyn Command>) -> Result<()> {
        self.pending_commands.push(command);
        Ok(())
    }

    pub fn execute_pending_commands(&mut self, resources: &mut Resources) -> Result<()> {
        self.pending_commands
            .drain(..)
            .collect::<Vec<_>>()
            .into_iter()
            .try_for_each(|command| self.execute(command, resources))
    }

    pub fn execute(
        &mut self,
        mut command: Box<dyn Command>,
        resources: &mut Resources,
    ) -> Result<()> {
        command.execute(resources)?;
        if command.is_undoable() {
            self.undo_commands.push(command);
            self.redo_commands.clear();
        }
        Ok(())
    }

    pub fn undo(&mut self, resources: &mut Resources) -> Result<()> {
        if let Some(mut command) = self.undo_commands.pop() {
            command.undo(resources)?;
            self.redo_commands.push(command);
        }
        Ok(())
    }

    pub fn redo(&mut self, resources: &mut Resources) -> Result<()> {
        if let Some(mut command) = self.redo_commands.pop() {
            command.execute(resources)?;
            self.undo_commands.push(command);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct LoadGltfAssetCommand(pub PathBuf);

impl Command for LoadGltfAssetCommand {
    fn is_undoable(&self) -> bool {
        false
    }

    fn execute(&mut self, resources: &mut Resources) -> Result<()> {
        log::info!("Loading GLTF Asset: {:?}", &self.0);
        resources.load_gltf_asset(&self.0).unwrap();
        Ok(())
    }

    fn undo(&mut self, _resources: &mut Resources) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct OpenMapCommand(pub PathBuf);

impl Command for OpenMapCommand {
    fn is_undoable(&self) -> bool {
        true
    }

    fn execute(&mut self, resources: &mut Resources) -> Result<()> {
        log::info!("Loading phantom map: {:?}", &self.0);
        resources.open_map(&self.0).unwrap();
        Ok(())
    }

    fn undo(&mut self, resources: &mut Resources) -> Result<()> {
        log::info!("Closing map: {:?}", &self.0);
        resources.close_map().unwrap();
        Ok(())
    }
}

#[derive(Debug)]
pub struct SaveMapCommand(pub PathBuf);

impl Command for SaveMapCommand {
    fn is_undoable(&self) -> bool {
        false
    }

    fn execute(&mut self, resources: &mut Resources) -> Result<()> {
        log::info!("Saving map: {:?}", &self.0);
        resources.world.save(&self.0).unwrap();
        Ok(())
    }

    fn undo(&mut self, _resources: &mut Resources) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct CloseMapCommand;

impl Command for CloseMapCommand {
    fn is_undoable(&self) -> bool {
        false
    }

    fn execute(&mut self, resources: &mut Resources) -> Result<()> {
        log::info!("Closing map");
        resources.close_map().unwrap();
        Ok(())
    }

    fn undo(&mut self, _resources: &mut Resources) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct ExitCommand;

impl Command for ExitCommand {
    fn is_undoable(&self) -> bool {
        false
    }

    fn execute(&mut self, resources: &mut Resources) -> Result<()> {
        log::info!("Exiting...");
        resources.system.exit_requested = true;
        Ok(())
    }

    fn undo(&mut self, _resources: &mut Resources) -> Result<()> {
        Ok(())
    }
}
