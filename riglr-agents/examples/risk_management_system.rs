//! Risk Management System Example
//!
//! This example demonstrates a comprehensive risk management system using
//! multiple coordinated agents:
//! - Position Risk Analyzer: Evaluates individual trade risks
//! - Portfolio Risk Monitor: Tracks overall portfolio exposure
//! - Compliance Agent: Ensures regulatory compliance
//! - Alert Manager: Handles risk alerts and notifications
//! - Emergency Response Agent: Takes corrective actions
//!
//! The system shows how multiple agents can work together to implement
//! sophisticated risk management strategies across different blockchain
//! networks and asset classes.
//!
//! Run with: cargo run --example risk_management_system

use riglr_agents::{
    Agent, AgentDispatcher, AgentRegistry, LocalAgentRegistry,
    Task, TaskResult, TaskType, Priority, AgentId, AgentMessage,
    DispatchConfig, RoutingStrategy, ChannelCommunication, AgentCommunication
};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::collections::HashMap;
use serde_json::json;
use tokio::time::sleep;

/// Risk limits and configuration
#[derive(Debug, Clone)]
struct RiskLimits {
    max_position_size: f64,
    max_daily_loss: f64,
    max_portfolio_var: f64,  // Value at Risk
    max_leverage: f64,
    max_correlation_exposure: f64,
    sector_limits: HashMap<String, f64>,
}

impl Default for RiskLimits {
    fn default() -> Self {
        let mut sector_limits = HashMap::new();
        sector_limits.insert("defi".to_string(), 0.30);
        sector_limits.insert("gaming".to_string(), 0.15);
        sector_limits.insert("infrastructure".to_string(), 0.25);
        sector_limits.insert("meme".to_string(), 0.10);
        
        Self {
            max_position_size: 0.20,     // 20% max per position
            max_daily_loss: 0.05,        // 5% max daily loss
            max_portfolio_var: 0.15,     // 15% max VaR
            max_leverage: 3.0,           // 3x max leverage
            max_correlation_exposure: 0.60, // 60% max in correlated assets
            sector_limits,
        }
    }
}

/// Current risk state of the portfolio
#[derive(Debug, Clone)]
struct RiskState {
    current_var: f64,
    daily_pnl: f64,
    total_exposure: f64,
    leverage_ratio: f64,
    sector_exposures: HashMap<String, f64>,
    position_risks: HashMap<String, f64>,
    correlation_matrix: HashMap<String, HashMap<String, f64>>,
    alert_level: AlertLevel,
}

#[derive(Debug, Clone, PartialEq)]
enum AlertLevel {
    Green,   // Normal operations
    Yellow,  // Elevated risk
    Orange,  // High risk - caution required
    Red,     // Critical risk - immediate action needed
}

impl Default for RiskState {
    fn default() -> Self {
        Self {
            current_var: 0.08,
            daily_pnl: -0.02,
            total_exposure: 0.75,
            leverage_ratio: 1.5,
            sector_exposures: HashMap::new(),
            position_risks: HashMap::new(),
            correlation_matrix: HashMap::new(),
            alert_level: AlertLevel::Green,
        }
    }
}

/// Position risk analysis agent
#[derive(Clone)]
struct PositionRiskAgent {
    id: AgentId,
    communication: Arc<ChannelCommunication>,
    risk_limits: Arc<RiskLimits>,
    risk_state: Arc<Mutex<RiskState>>,
}

impl PositionRiskAgent {
    fn new(
        id: &str,
        communication: Arc<ChannelCommunication>,
        risk_limits: Arc<RiskLimits>,
        risk_state: Arc<Mutex<RiskState>>
    ) -> Self {
        Self {
            id: AgentId::new(id),
            communication,
            risk_limits,
            risk_state,
        }
    }

    async fn analyze_position_risk(&self, position_data: &serde_json::Value) -> serde_json::Value {
        let symbol = position_data.get("symbol")
            .and_then(|s| s.as_str())
            .unwrap_or("UNKNOWN");
        
        let size = position_data.get("size")
            .and_then(|s| s.as_f64())
            .unwrap_or(0.0);
        
        let price = position_data.get("current_price")
            .and_then(|p| p.as_f64())
            .unwrap_or(1.0);
        
        let volatility = position_data.get("volatility")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.25);
        
        // Calculate various risk metrics
        let position_value = size * price;
        let var_1day = position_value * volatility * 2.33; // 99% VaR
        let max_drawdown = position_value * volatility * 3.0; // Estimated max drawdown
        
        let risk_state = self.risk_state.lock().unwrap();
        let position_percent = position_value / (risk_state.total_exposure * 100000.0); // Assuming 100k portfolio
        
        let risk_score = {
            let size_risk = position_percent / self.risk_limits.max_position_size;
            let volatility_risk = volatility / 0.5; // Normalize against 50% volatility
            let liquidity_risk = 1.0; // Simplified - would use actual liquidity data
            
            (size_risk + volatility_risk + liquidity_risk) / 3.0
        };
        
        let approved = risk_score < 1.0 && position_percent < self.risk_limits.max_position_size;
        
        json!({
            "symbol": symbol,
            "position_value": position_value,
            "position_percent": position_percent,
            "var_1day": var_1day,
            "max_drawdown": max_drawdown,
            "risk_score": risk_score,
            "volatility": volatility,
            "approved": approved,
            "risk_level": match risk_score {
                s if s > 1.5 => "critical",
                s if s > 1.0 => "high",
                s if s > 0.7 => "medium",
                _ => "low"
            },
            "recommendations": if approved {
                vec!["APPROVE", "MONITOR"]
            } else {
                vec!["REJECT", "REDUCE_SIZE", "ADD_HEDGING"]
            },
            "analyzer": self.id.as_str(),
            "timestamp": chrono::Utc::now().timestamp()
        })
    }
}

#[async_trait]
impl Agent for PositionRiskAgent {
    async fn execute_task(&self, task: Task) -> riglr_agents::Result<TaskResult> {
        println!("🎯 Position Risk Agent {} analyzing position", self.id);
        
        let position_data = task.parameters.get("position")
            .ok_or_else(|| riglr_agents::AgentError::task_execution("Missing position data"))?;
        
        let risk_analysis = self.analyze_position_risk(position_data).await;
        
        // Update risk state with new position data
        {
            let mut risk_state = self.risk_state.lock().unwrap();
            let symbol = position_data.get("symbol")
                .and_then(|s| s.as_str())
                .unwrap_or("UNKNOWN");
            let risk_score = risk_analysis.get("risk_score")
                .and_then(|r| r.as_f64())
                .unwrap_or(0.0);
            let position_percent = risk_analysis.get("position_percent")
                .and_then(|p| p.as_f64())
                .unwrap_or(0.0);
                
            risk_state.position_risks.insert(symbol.to_string(), risk_score);
            
            // Update sector exposures if sector information is available
            if let Some(sector) = position_data.get("sector").and_then(|s| s.as_str()) {
                let current_sector_exposure = risk_state.sector_exposures.get(sector).copied().unwrap_or(0.0);
                risk_state.sector_exposures.insert(sector.to_string(), current_sector_exposure + position_percent);
            }
            
            // Update correlation matrix with simple correlation assumptions
            // In practice, this would use real correlation data
            if symbol.contains("BTC") || symbol.contains("ETH") {
                let existing_symbols: Vec<String> = risk_state.position_risks.keys().cloned().collect();
                for existing_symbol in existing_symbols {
                    if existing_symbol != symbol && (existing_symbol.contains("BTC") || existing_symbol.contains("ETH")) {
                        risk_state.correlation_matrix
                            .entry(symbol.to_string())
                            .or_default()
                            .insert(existing_symbol, 0.8); // High correlation for crypto
                    }
                }
            }
        }
        
        // Broadcast risk analysis to other agents
        let message = AgentMessage::new(
            self.id.clone(),
            None,
            "position_risk_analysis".to_string(),
            risk_analysis.clone()
        );
        
        self.communication.broadcast_message(message).await
            .map_err(|e| riglr_agents::AgentError::communication(e.to_string()))?;
        
        Ok(TaskResult::success(
            risk_analysis,
            None,
            Duration::from_millis(150)
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "position_risk_analysis".to_string(),
            "position_analysis".to_string(),
            "volatility_analysis".to_string(),
            "var_calculation".to_string(),
        ]
    }
}

/// Portfolio-level risk monitoring agent
#[derive(Clone)]
struct PortfolioRiskMonitor {
    id: AgentId,
    communication: Arc<ChannelCommunication>,
    risk_limits: Arc<RiskLimits>,
    risk_state: Arc<Mutex<RiskState>>,
}

impl PortfolioRiskMonitor {
    fn new(
        id: &str,
        communication: Arc<ChannelCommunication>,
        risk_limits: Arc<RiskLimits>,
        risk_state: Arc<Mutex<RiskState>>
    ) -> Self {
        Self {
            id: AgentId::new(id),
            communication,
            risk_limits,
            risk_state,
        }
    }

    async fn calculate_portfolio_risk(&self) -> serde_json::Value {
        let mut risk_state = self.risk_state.lock().unwrap();
        
        // Simulate portfolio risk calculations
        let total_var = risk_state.position_risks.values().sum::<f64>();
        let correlation_adjustment = 0.85; // Account for correlations
        risk_state.current_var = total_var * correlation_adjustment;
        
        // Check correlation exposure limits
        let correlation_exposure = self.calculate_correlation_exposure(&risk_state);
        
        // Check sector limits
        let sector_violations = self.check_sector_limits(&risk_state);
        
        // Calculate alert level
        let new_alert_level = if risk_state.current_var > self.risk_limits.max_portfolio_var {
            AlertLevel::Red
        } else if risk_state.current_var > self.risk_limits.max_portfolio_var * 0.8 {
            AlertLevel::Orange
        } else if risk_state.current_var > self.risk_limits.max_portfolio_var * 0.6 {
            AlertLevel::Yellow
        } else {
            AlertLevel::Green
        };
        
        let alert_changed = risk_state.alert_level != new_alert_level;
        risk_state.alert_level = new_alert_level.clone();
        
        // Check various risk limits
        let mut limit_breaches = vec![
            if risk_state.current_var > self.risk_limits.max_portfolio_var {
                Some("PORTFOLIO_VAR_EXCEEDED")
            } else { None },
            if risk_state.daily_pnl.abs() > self.risk_limits.max_daily_loss {
                Some("DAILY_LOSS_LIMIT_EXCEEDED")
            } else { None },
            if risk_state.leverage_ratio > self.risk_limits.max_leverage {
                Some("LEVERAGE_LIMIT_EXCEEDED")
            } else { None },
            if correlation_exposure > self.risk_limits.max_correlation_exposure {
                Some("CORRELATION_EXPOSURE_EXCEEDED")
            } else { None },
        ].into_iter().flatten().collect::<Vec<_>>();
        
        // Add sector limit violations
        limit_breaches.extend(sector_violations.iter().map(|s| s.as_str()));
        
        let risk_summary = json!({
            "portfolio_var": risk_state.current_var,
            "daily_pnl": risk_state.daily_pnl,
            "total_exposure": risk_state.total_exposure,
            "leverage_ratio": risk_state.leverage_ratio,
            "correlation_exposure": correlation_exposure,
            "alert_level": format!("{:?}", risk_state.alert_level),
            "alert_changed": alert_changed,
            "limit_breaches": limit_breaches,
            "risk_limits": {
                "max_var": self.risk_limits.max_portfolio_var,
                "max_daily_loss": self.risk_limits.max_daily_loss,
                "max_leverage": self.risk_limits.max_leverage,
                "max_correlation_exposure": self.risk_limits.max_correlation_exposure
            },
            "sector_exposures": risk_state.sector_exposures,
            "sector_limits": self.risk_limits.sector_limits,
            "position_count": risk_state.position_risks.len(),
            "recommendations": match risk_state.alert_level {
                AlertLevel::Red => vec!["REDUCE_EXPOSURE", "HEDGE_PORTFOLIO", "CLOSE_HIGH_RISK_POSITIONS"],
                AlertLevel::Orange => vec!["MONITOR_CLOSELY", "PREPARE_HEDGES", "REVIEW_POSITIONS"],
                AlertLevel::Yellow => vec!["INCREASE_MONITORING", "REVIEW_CORRELATIONS"],
                AlertLevel::Green => vec!["MAINTAIN_CURRENT_STRATEGY"],
            },
            "monitor": self.id.as_str(),
            "timestamp": chrono::Utc::now().timestamp()
        });
        
        risk_summary
    }
    
    fn calculate_correlation_exposure(&self, risk_state: &RiskState) -> f64 {
        // Simulate correlation exposure calculation
        // In a real system, this would use the correlation matrix to calculate
        // the exposure to highly correlated assets
        let mut correlated_exposure = 0.0;
        
        // Simple simulation: assume crypto assets are highly correlated
        for (symbol, risk) in &risk_state.position_risks {
            if symbol.contains("BTC") || symbol.contains("ETH") || symbol.contains("crypto") {
                correlated_exposure += risk;
            }
        }
        
        // Also use correlation matrix if available
        if !risk_state.correlation_matrix.is_empty() {
            // In practice, this would compute portfolio correlation exposure
            // using the correlation matrix and position weights
        }
        
        correlated_exposure.min(1.0) // Cap at 100%
    }
    
    fn check_sector_limits(&self, risk_state: &RiskState) -> Vec<String> {
        let mut violations = Vec::new();
        
        for (sector, current_exposure) in &risk_state.sector_exposures {
            if let Some(limit) = self.risk_limits.sector_limits.get(sector) {
                if current_exposure > limit {
                    violations.push(format!("SECTOR_LIMIT_EXCEEDED_{}", sector.to_uppercase()));
                }
            }
        }
        
        violations
    }
}

#[async_trait]
impl Agent for PortfolioRiskMonitor {
    async fn execute_task(&self, _task: Task) -> riglr_agents::Result<TaskResult> {
        println!("📊 Portfolio Risk Monitor {} calculating portfolio risk", self.id);
        
        let risk_summary = self.calculate_portfolio_risk().await;
        
        // If alert level changed or we have breaches, send high-priority message
        let alert_changed = risk_summary.get("alert_changed")
            .and_then(|c| c.as_bool())
            .unwrap_or(false);
        
        let has_breaches = risk_summary.get("limit_breaches")
            .and_then(|b| b.as_array())
            .map(|arr| !arr.is_empty())
            .unwrap_or(false);
        
        if alert_changed || has_breaches {
            let alert_message = AgentMessage::new(
                self.id.clone(),
                None,
                "risk_alert".to_string(),
                risk_summary.clone()
            );
            
            self.communication.broadcast_message(alert_message).await
                .map_err(|e| riglr_agents::AgentError::communication(e.to_string()))?;
        }
        
        // Regular portfolio status update
        let status_message = AgentMessage::new(
            self.id.clone(),
            None,
            "portfolio_risk_update".to_string(),
            risk_summary.clone()
        );
        
        self.communication.broadcast_message(status_message).await
            .map_err(|e| riglr_agents::AgentError::communication(e.to_string()))?;
        
        Ok(TaskResult::success(
            risk_summary,
            None,
            Duration::from_millis(200)
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "portfolio".to_string(),
            "risk_monitoring".to_string(),
            "alert_management".to_string(),
            "limit_monitoring".to_string(),
        ]
    }
}

/// Compliance checking agent
#[derive(Clone)]
struct ComplianceAgent {
    id: AgentId,
    communication: Arc<ChannelCommunication>,
    regulatory_rules: HashMap<String, serde_json::Value>,
}

impl ComplianceAgent {
    fn new(id: &str, communication: Arc<ChannelCommunication>) -> Self {
        let mut regulatory_rules = HashMap::new();
        
        // Example regulatory rules
        regulatory_rules.insert("max_single_issuer".to_string(), json!(0.25));
        regulatory_rules.insert("min_liquidity_ratio".to_string(), json!(0.10));
        regulatory_rules.insert("prohibited_sectors".to_string(), json!(["gambling", "adult_content"]));
        regulatory_rules.insert("kyc_required_threshold".to_string(), json!(50000.0));
        
        Self {
            id: AgentId::new(id),
            communication,
            regulatory_rules,
        }
    }

    async fn check_compliance(&self, trade_data: &serde_json::Value) -> serde_json::Value {
        sleep(Duration::from_millis(50)).await;
        
        let symbol = trade_data.get("symbol")
            .and_then(|s| s.as_str())
            .unwrap_or("UNKNOWN");
        
        let amount = trade_data.get("amount")
            .and_then(|a| a.as_f64())
            .unwrap_or(0.0);
        
        let sector = trade_data.get("sector")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown");
        
        // Check various compliance rules
        let mut violations = Vec::new();
        let mut warnings = Vec::new();
        
        // Check prohibited sectors
        if let Some(prohibited) = self.regulatory_rules.get("prohibited_sectors") {
            if let Some(sectors) = prohibited.as_array() {
                if sectors.iter().any(|s| s.as_str() == Some(sector)) {
                    violations.push(format!("PROHIBITED_SECTOR: {}", sector));
                }
            }
        }
        
        // Check KYC threshold
        if let Some(kyc_threshold) = self.regulatory_rules.get("kyc_required_threshold") {
            if let Some(threshold) = kyc_threshold.as_f64() {
                if amount * 50000.0 > threshold { // Assuming $50k price
                    warnings.push("KYC_VERIFICATION_REQUIRED".to_string());
                }
            }
        }
        
        let compliant = violations.is_empty();
        
        json!({
            "symbol": symbol,
            "amount": amount,
            "sector": sector,
            "compliant": compliant,
            "violations": violations,
            "warnings": warnings,
            "regulatory_framework": "US_SEC_COMPLIANT",
            "compliance_score": if compliant { 1.0 } else { 0.0 },
            "required_actions": if !violations.is_empty() {
                vec!["REJECT_TRADE", "LEGAL_REVIEW"]
            } else if !warnings.is_empty() {
                vec!["ADDITIONAL_VERIFICATION", "DOCUMENTATION"]
            } else {
                vec!["NONE"]
            },
            "checker": self.id.as_str(),
            "timestamp": chrono::Utc::now().timestamp()
        })
    }
}

#[async_trait]
impl Agent for ComplianceAgent {
    async fn execute_task(&self, task: Task) -> riglr_agents::Result<TaskResult> {
        println!("⚖️ Compliance Agent {} checking regulatory compliance", self.id);
        
        let trade_data = task.parameters.get("trade_data")
            .ok_or_else(|| riglr_agents::AgentError::task_execution("Missing trade data"))?;
        
        let compliance_result = self.check_compliance(trade_data).await;
        
        // If there are violations, send immediate alert
        let has_violations = compliance_result.get("violations")
            .and_then(|v| v.as_array())
            .map(|arr| !arr.is_empty())
            .unwrap_or(false);
        
        if has_violations {
            let alert_message = AgentMessage::new(
                self.id.clone(),
                None,
                "compliance_violation".to_string(),
                compliance_result.clone()
            );
            
            self.communication.broadcast_message(alert_message).await
                .map_err(|e| riglr_agents::AgentError::communication(e.to_string()))?;
        }
        
        Ok(TaskResult::success(
            compliance_result,
            None,
            Duration::from_millis(50)
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "compliance_risk_analysis".to_string(),
            "compliance".to_string(),
            "regulatory_check".to_string(),
            "legal_validation".to_string(),
        ]
    }
}

/// Emergency response agent that takes corrective actions
#[derive(Clone)]
struct EmergencyResponseAgent {
    id: AgentId,
    communication: Arc<ChannelCommunication>,
    risk_state: Arc<Mutex<RiskState>>,
}

impl EmergencyResponseAgent {
    fn new(
        id: &str,
        communication: Arc<ChannelCommunication>,
        risk_state: Arc<Mutex<RiskState>>
    ) -> Self {
        Self {
            id: AgentId::new(id),
            communication,
            risk_state,
        }
    }

    async fn execute_emergency_response(&self, alert_data: &serde_json::Value) -> serde_json::Value {
        let alert_level = alert_data.get("alert_level")
            .and_then(|l| l.as_str())
            .unwrap_or("Green");
        
        let default_breaches = vec![];
        let limit_breaches = alert_data.get("limit_breaches")
            .and_then(|b| b.as_array())
            .unwrap_or(&default_breaches)
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>();
        
        let mut actions_taken = Vec::new();
        
        match alert_level {
            "Red" => {
                // Critical risk - immediate action required
                println!("🚨 CRITICAL RISK DETECTED - Taking emergency actions");
                
                // Simulate emergency position reduction
                actions_taken.push("REDUCED_HIGH_RISK_POSITIONS");
                actions_taken.push("ACTIVATED_PORTFOLIO_HEDGING");
                actions_taken.push("SUSPENDED_NEW_TRADING");
                
                // Update risk state to reflect emergency actions
                {
                    let mut risk_state = self.risk_state.lock().unwrap();
                    risk_state.total_exposure *= 0.7; // Reduce exposure by 30%
                    risk_state.current_var *= 0.6;    // Reduce VaR through hedging
                }
                
                sleep(Duration::from_millis(100)).await;
            }
            "Orange" => {
                // High risk - precautionary measures
                println!("⚠️ HIGH RISK - Taking precautionary measures");
                
                actions_taken.push("INCREASED_MONITORING");
                actions_taken.push("PREPARED_HEDGING_POSITIONS");
                actions_taken.push("ALERTED_RISK_MANAGEMENT_TEAM");
                
                sleep(Duration::from_millis(50)).await;
            }
            "Yellow" => {
                // Elevated risk - enhanced monitoring
                println!("📊 ELEVATED RISK - Enhanced monitoring activated");
                
                actions_taken.push("ENHANCED_MONITORING");
                actions_taken.push("UPDATED_RISK_PARAMETERS");
                
                sleep(Duration::from_millis(25)).await;
            }
            _ => {
                // Normal operations
                actions_taken.push("NORMAL_MONITORING");
            }
        }
        
        // Handle specific limit breaches
        for breach in &limit_breaches {
            match *breach {
                "PORTFOLIO_VAR_EXCEEDED" => {
                    actions_taken.push("VAR_LIMIT_BREACH_RESPONSE");
                    actions_taken.push("REDUCED_PORTFOLIO_VOLATILITY");
                }
                "DAILY_LOSS_LIMIT_EXCEEDED" => {
                    actions_taken.push("DAILY_LOSS_RESPONSE");
                    actions_taken.push("SUSPENDED_RISKY_STRATEGIES");
                }
                "LEVERAGE_LIMIT_EXCEEDED" => {
                    actions_taken.push("DELEVERAGING_POSITIONS");
                }
                "CORRELATION_EXPOSURE_EXCEEDED" => {
                    actions_taken.push("CORRELATION_LIMIT_BREACH_RESPONSE");
                    actions_taken.push("DIVERSIFIED_CORRELATED_POSITIONS");
                }
                breach_type if breach_type.starts_with("SECTOR_LIMIT_EXCEEDED_") => {
                    actions_taken.push("SECTOR_LIMIT_BREACH_RESPONSE");
                    actions_taken.push("REDUCED_SECTOR_CONCENTRATION");
                }
                _ => {}
            }
        }
        
        json!({
            "alert_level": alert_level,
            "response_time": chrono::Utc::now().timestamp(),
            "actions_taken": actions_taken,
            "limit_breaches_handled": limit_breaches,
            "response_effectiveness": "HIGH", // Would be calculated based on actual results
            "follow_up_required": !actions_taken.is_empty() && actions_taken != vec!["NORMAL_MONITORING"],
            "responder": self.id.as_str(),
            "next_review": chrono::Utc::now().timestamp() + 3600, // 1 hour
            "incident_id": uuid::Uuid::new_v4().to_string()
        })
    }
}

#[async_trait]
impl Agent for EmergencyResponseAgent {
    async fn execute_task(&self, task: Task) -> riglr_agents::Result<TaskResult> {
        println!("🚨 Emergency Response Agent {} handling risk alert", self.id);
        
        let alert_data = task.parameters.get("alert_data")
            .ok_or_else(|| riglr_agents::AgentError::task_execution("Missing alert data"))?;
        
        let response = self.execute_emergency_response(alert_data).await;
        
        // Notify all agents of emergency response actions
        let message = AgentMessage::new(
            self.id.clone(),
            None,
            "emergency_response_executed".to_string(),
            response.clone()
        );
        
        self.communication.broadcast_message(message).await
            .map_err(|e| riglr_agents::AgentError::communication(e.to_string()))?;
        
        Ok(TaskResult::success(
            response,
            None,
            Duration::from_millis(200)
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "monitoring".to_string(),
            "emergency_response".to_string(),
            "risk_mitigation".to_string(),
            "incident_management".to_string(),
        ]
    }

    async fn handle_message(&self, message: AgentMessage) -> riglr_agents::Result<()> {
        match message.message_type.as_str() {
            "risk_alert" => {
                println!("🚨 Emergency Response Agent {} received risk alert", self.id);
                // Could automatically trigger emergency response task
            }
            "compliance_violation" => {
                println!("🚨 Emergency Response Agent {} received compliance violation alert", self.id);
                // Could trigger immediate compliance response
            }
            _ => {}
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 Starting Risk Management System Example");
    println!("🛡️ Initializing comprehensive risk management framework");
    
    // Initialize risk configuration and state
    let risk_limits = Arc::new(RiskLimits::default());
    let risk_state = Arc::new(Mutex::new(RiskState::default()));
    
    // Initialize communication system
    let communication = Arc::new(ChannelCommunication::new());
    
    // Create specialized risk management agents
    let position_risk_agent = Arc::new(PositionRiskAgent::new("risk-analyzer-1", communication.clone(), risk_limits.clone(), risk_state.clone()));
    let portfolio_monitor = Arc::new(PortfolioRiskMonitor::new("portfolio-monitor-1", communication.clone(), risk_limits.clone(), risk_state.clone()));
    let compliance_agent = Arc::new(ComplianceAgent::new("compliance-1", communication.clone()));
    let emergency_agent = Arc::new(EmergencyResponseAgent::new("emergency-response-1", communication.clone(), risk_state.clone()));
    
    // Create agent registry and register all agents
    let registry = Arc::new(LocalAgentRegistry::new());
    registry.register_agent(position_risk_agent.clone()).await?;
    registry.register_agent(portfolio_monitor.clone()).await?;
    registry.register_agent(compliance_agent.clone()).await?;
    registry.register_agent(emergency_agent.clone()).await?;
    
    let agents = registry.list_agents().await?;
    println!("✅ Registered {} risk management agents", agents.len());
    
    // Create dispatcher optimized for risk workflows
    let dispatch_config = DispatchConfig {
        routing_strategy: RoutingStrategy::Capability,
        max_retries: 1, // Risk decisions should be fast
        default_task_timeout: Duration::from_secs(15),
        retry_delay: Duration::from_millis(500),
        max_concurrent_tasks_per_agent: 5,
        enable_load_balancing: false,
    };
    
    let dispatcher = Arc::new(AgentDispatcher::with_config(registry.clone(), dispatch_config));
    
    // Execute comprehensive risk management workflow
    {
        println!("\n🔄 Starting Risk Management Workflow");
        
        // Scenario 1: Normal position risk analysis
        println!("\n1️⃣ Scenario 1: Normal Position Risk Analysis");
        let position_task = Task::new(
            TaskType::Custom("position_risk_analysis".to_string()),
            json!({
                "position": {
                    "symbol": "BTC",
                    "size": 2.5,
                    "current_price": 50000.0,
                    "volatility": 0.18,
                    "sector": "defi",
                    "liquidity_score": 0.95
                }
            })
        ).with_priority(Priority::High);
        
        let position_result = dispatcher.dispatch_task(position_task).await?;
        println!("✅ Position risk analysis: {} (Risk Score: {:.2})", 
            if position_result.data().unwrap().get("approved").and_then(|a| a.as_bool()).unwrap_or(false) { 
                "APPROVED" 
            } else { 
                "REJECTED" 
            },
            position_result.data().unwrap().get("risk_score").and_then(|s| s.as_f64()).unwrap_or(0.0)
        );
        
        sleep(Duration::from_millis(100)).await;
        
        // Scenario 2: Portfolio risk monitoring
        println!("\n2️⃣ Scenario 2: Portfolio Risk Assessment");
        let portfolio_task = Task::new(
            TaskType::Portfolio,
            json!({
                "analysis_type": "comprehensive",
                "include_stress_test": true
            })
        ).with_priority(Priority::High);
        
        let portfolio_result = dispatcher.dispatch_task(portfolio_task).await?;
        let alert_level = portfolio_result.data().unwrap().get("alert_level")
            .and_then(|l| l.as_str())
            .unwrap_or("Green");
        println!("✅ Portfolio risk assessment: Alert Level {}", alert_level);
        
        sleep(Duration::from_millis(100)).await;
        
        // Scenario 3: Compliance checking
        println!("\n3️⃣ Scenario 3: Regulatory Compliance Check");
        let compliance_task = Task::new(
            TaskType::Custom("compliance_risk_analysis".to_string()),
            json!({
                "trade_data": {
                    "symbol": "ETH",
                    "amount": 10.0,
                    "sector": "defi",
                    "counterparty": "binance",
                    "jurisdiction": "US"
                }
            })
        ).with_priority(Priority::High);
        
        let compliance_result = dispatcher.dispatch_task(compliance_task).await?;
        let compliant = compliance_result.data().unwrap().get("compliant")
            .and_then(|c| c.as_bool())
            .unwrap_or(false);
        println!("✅ Compliance check: {}", if compliant { "COMPLIANT" } else { "NON-COMPLIANT" });
        
        sleep(Duration::from_millis(100)).await;
        
        // Scenario 4: Emergency response simulation
        println!("\n4️⃣ Scenario 4: Emergency Response Simulation");
        
        // Simulate a high-risk scenario
        {
            let mut risk_state = risk_state.lock().unwrap();
            risk_state.current_var = 0.20; // Exceed VaR limit
            risk_state.daily_pnl = -0.08;  // Exceed daily loss limit
            risk_state.alert_level = AlertLevel::Red;
        }
        
        let emergency_task = Task::new(
            TaskType::Monitoring,
            json!({
                "alert_data": {
                    "alert_level": "Red",
                    "limit_breaches": ["PORTFOLIO_VAR_EXCEEDED", "DAILY_LOSS_LIMIT_EXCEEDED"],
                    "severity": "critical",
                    "immediate_action_required": true
                }
            })
        ).with_priority(Priority::Critical);
        
        let emergency_result = dispatcher.dispatch_task(emergency_task).await?;
        let actions_taken = emergency_result.data().unwrap().get("actions_taken")
            .and_then(|a| a.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);
        println!("✅ Emergency response executed: {} corrective actions taken", actions_taken);
        
        sleep(Duration::from_millis(200)).await;
        
        // Final portfolio state after emergency response
        println!("\n5️⃣ Scenario 5: Post-Emergency Portfolio Assessment");
        let final_portfolio_task = Task::new(
            TaskType::Portfolio,
            json!({
                "analysis_type": "post_emergency",
                "validate_actions": true
            })
        ).with_priority(Priority::High);
        
        let final_result = dispatcher.dispatch_task(final_portfolio_task).await?;
        let final_alert = final_result.data().unwrap().get("alert_level")
            .and_then(|l| l.as_str())
            .unwrap_or("Unknown");
        println!("✅ Post-emergency assessment: Alert Level {}", final_alert);
        
    }
    
    // Final system status
    println!("\n📊 Risk Management System Status:");
    let agents = registry.list_agents().await?;
    for agent in agents {
        let status = agent.status();
        println!("  {} - {} capabilities", 
            status.agent_id,
            status.capabilities.len()
        );
    }
    
    // Display final risk state
    {
        let risk_state = risk_state.lock().unwrap();
        println!("\n🛡️ Final Risk State:");
        println!("  Portfolio VaR: {:.2}% (Limit: {:.2}%)", 
            risk_state.current_var * 100.0, risk_limits.max_portfolio_var * 100.0);
        println!("  Daily P&L: {:.2}% (Limit: {:.2}%)", 
            risk_state.daily_pnl * 100.0, risk_limits.max_daily_loss * 100.0);
        println!("  Alert Level: {:?}", risk_state.alert_level);
        println!("  Positions Monitored: {}", risk_state.position_risks.len());
    }
    
    println!("\n🎉 Risk Management System example completed successfully!");
    println!("The system demonstrated:");
    println!("  - Position-level risk analysis with VaR calculations");
    println!("  - Portfolio-wide risk monitoring and alerting"); 
    println!("  - Regulatory compliance checking");
    println!("  - Automated emergency response procedures");
    println!("  - Real-time coordination between specialized agents");
    
    Ok(())
}