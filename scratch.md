# Branches
- Phase7
- random-qa
- phaseA

---

  8 phases with concrete file paths:
  - A: Core types & schema
  - B: Stack/Card CRUD + Tauri commands
  - C: Execution engine + sandbox
  - D: Artifact system
  - E: Agent tools & prompts
  - F: Frontend - Stack/Card UI
  - G: Frontend - Artifacts & execution views
  - H: Frontend - Guided setup UX

---
Gemini: AIzaSyAS2V5Yux1oRLZeytHBvz8KuBk_sVVwuUU
---

Phase 1 and 2 done

Start on Phase 3

Implement Phase 4: Execution Engine for LEAF-RS                                                                                                       
                                                                                                                                                        
  Phases 1-3 are complete. Now implement Phase 4: Execution Engine (Deno sandbox).                                                                      
                                                                                                                                                        
  Reference these docs:                                                                                                                                 
  - @context/CONCEPTS.md - Concepts & Synchronizations model (READ FIRST)                                                                               
  - @context/RUST_REWRITE_PLAN.md - Full implementation plan                                                                                            
  - @CLAUDE.md - Project context                                                                                                                        
                                                                                                                                                        
  Phase 4 requirements from the plan:                                                                                                                   
  1. Bundle or download Deno runtime                                                                                                                    
  2. Implement leaf-executor crate with sandbox wrapper                                                                                                 
  3. Execution lifecycle (start, complete, fail)                                                                                                        
  4. Retry logic with configurable attempts                                                                                                             
  5. stdout/stderr capture                                                                                                                              
  6. Execution history UI                                                                                                                               
                                                                                                                                                        
  Key synchronizations to implement:                                                                                                                    
  - Trigger.fire(card, event) → Execution.start(card, event)                                                                                            
  - Execution.start → Sandbox.setup → Sandbox.run                                                                                                       
  - Sandbox.run.success → Execution.complete                                                                                                            
  - Sandbox.run.failure → Execution.fail → retry logic                                                                                                  
                                                                                                                                                        
  The trigger_card command in leaf-app/src/commands/cards.rs has a TODO placeholder where execution should be triggered. Wire it up to the new executor.
                                                                                                                                                        
  Follow the Concepts & Synchronizations model - check the "Implementation Checklist" in CONCEPTS.md before marking complete.                           
                              