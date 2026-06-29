//! Monitoramento otimizado de processos usando sysinfo
//! Implementa estratégia híbrida: polling rápido (1s) aguardando jogo,
//! polling lento (10s) quando jogo está rodando

use std::time::Duration;
use sysinfo::{Pid, ProcessRefreshKind, System};

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

/// Monitor otimizado de processos
pub struct ProcessMonitor {
    system: System,
    target_name_lower: String, // Pré-computado lowercase
    cached_pid: Option<Pid>,
    state: ProcessState,
}

impl ProcessMonitor {
    /// Cria novo monitor para um processo específico
    ///
    /// # Arguments
    /// * `process_name` - Nome do processo (ex: "game.exe", "elden ring.exe")
    pub fn new(process_name: String) -> Self {
        Self {
            system: System::new(), // Lazy initialization (vazio)
            target_name_lower: process_name.to_lowercase(),
            cached_pid: None,
            state: ProcessState::Waiting,
        }
    }

    /// Verifica estado atual do processo
    /// Retorna ProcessState indicando se processo foi detectado, está rodando ou foi fechado
    pub fn check_process(&mut self) -> ProcessState {
        match self.state {
            ProcessState::Waiting => {
                // Fase 1: Busca rápida por novo processo (polling 1s)
                // Usa refresh mínimo: apenas lista de processos, sem dados extras
                self.system
                    .refresh_processes_specifics(ProcessRefreshKind::new());

                // Early exit: para assim que encontrar processo
                if let Some((pid, _)) = self
                    .system
                    .processes()
                    .iter()
                    .find(|(_, p)| p.name().to_lowercase() == self.target_name_lower)
                {
                    self.cached_pid = Some(*pid);
                    self.state = ProcessState::Running;
                    return ProcessState::Running;
                }
                ProcessState::Waiting
            }
            ProcessState::Running => {
                // Fase 2: Check ultra-rápido de PID cacheado (polling 10s)
                if let Some(pid) = self.cached_pid {
                    // Refresh apenas 1 processo (praticamente zero custo)
                    self.system.refresh_process(pid);

                    if self.system.process(pid).is_some() {
                        // Processo ainda está rodando
                        return ProcessState::Running;
                    }

                    // Processo morreu
                    self.cached_pid = None;
                    self.state = ProcessState::Stopped;
                    return ProcessState::Stopped;
                }
                ProcessState::Stopped
            }
            ProcessState::Stopped => {
                // Retorna para Waiting para buscar novo processo
                self.state = ProcessState::Waiting;
                ProcessState::Waiting
            }
        }
    }

    /// Retorna intervalo recomendado de polling para estado atual
    /// - Waiting: 1s (detecção rápida)
    /// - Running: 10s (economia máxima, só detectar fechamento)
    pub fn get_poll_interval(&self) -> Duration {
        match self.state {
            ProcessState::Waiting => Duration::from_secs(1), // Latência baixa
            ProcessState::Running => Duration::from_secs(10), // Economia máxima
            ProcessState::Stopped => Duration::from_secs(1), // Volta para waiting
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
        assert_eq!(monitor.get_poll_interval(), Duration::from_secs(1));
    }

    #[test]
    fn test_lowercase_comparison() {
        let monitor = ProcessMonitor::new("Game.EXE".to_string());
        assert_eq!(monitor.target_name_lower, "game.exe");
    }
}
