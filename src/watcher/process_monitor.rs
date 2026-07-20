#![allow(dead_code)]
//! Monitoramento otimizado de processos usando sysinfo
//! Implementa estratégia híbrida: polling rápido (1s) aguardando jogo,
//! polling lento (10s) quando jogo está rodando

use std::time::Duration;
use sysinfo::{ProcessesToUpdate, System};

/// Estado do monitoramento de processo
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcessState {
    /// Aguardando processo iniciar
    Waiting,
    /// Processo detectado e rodando
    Running,
    /// Processo foi fechado
    Stopped,
}

/// Monitor otimizado de processos com backoff adaptativo
pub struct ProcessMonitor {
    system: System,
    target_name_lower: String,
    state: ProcessState,
    consecutive_misses: u32,
}

impl ProcessMonitor {
    /// Cria novo monitor para um processo específico
    ///
    /// # Arguments
    /// * `process_name` - Nome do processo (ex: "game.exe", "elden ring.exe")
    pub fn new(process_name: String) -> Self {
        Self {
            system: System::new(),
            target_name_lower: process_name.to_lowercase(),
            state: ProcessState::Waiting,
            consecutive_misses: 0,
        }
    }

    /// Verifica estado atual do processo
    /// Retorna ProcessState indicando se processo foi detectado, está rodando ou foi fechado
    pub fn check_process(&mut self) -> ProcessState {
        self.system.refresh_processes(ProcessesToUpdate::All, true);

        let found = self
            .system
            .processes()
            .values()
            .any(|p| p.name().to_string_lossy().to_lowercase() == self.target_name_lower);

        match self.state {
            ProcessState::Waiting => {
                if found {
                    self.consecutive_misses = 0;
                    self.state = ProcessState::Running;
                    ProcessState::Running
                } else {
                    self.consecutive_misses += 1;
                    ProcessState::Waiting
                }
            }
            ProcessState::Running => {
                if found {
                    ProcessState::Running
                } else {
                    self.consecutive_misses = 0;
                    self.state = ProcessState::Stopped;
                    ProcessState::Stopped
                }
            }
            ProcessState::Stopped => {
                self.state = ProcessState::Waiting;
                ProcessState::Waiting
            }
        }
    }

    pub fn get_poll_interval(&self) -> Duration {
        match self.state {
            ProcessState::Waiting => {
                let base_ms: u64 = 500;
                let max_ms: u64 = 30_000;
                let factor = 1u64 << self.consecutive_misses.min(6);
                let ms = (base_ms.saturating_mul(factor)).min(max_ms);
                Duration::from_millis(ms)
            }
            ProcessState::Running => Duration::from_secs(5),
            ProcessState::Stopped => Duration::from_millis(500),
        }
    }

    /// Retorna estado atual (usado em testes)
    #[cfg(test)]
    pub fn current_state(&self) -> ProcessState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_monitor_creation() {
        let monitor = ProcessMonitor::new("test.exe".to_string());
        assert_eq!(monitor.current_state(), ProcessState::Waiting);
        assert_eq!(monitor.get_poll_interval(), Duration::from_millis(500));
    }

    #[test]
    fn test_lowercase_comparison() {
        let monitor = ProcessMonitor::new("Game.EXE".to_string());
        assert_eq!(monitor.target_name_lower, "game.exe");
    }
}
