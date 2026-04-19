# General Bots Security Policy

## Overview

This comprehensive security policy establishes the framework for protecting General Bots systems, data, and operations. It covers information security, access control, data protection, incident response, and ongoing maintenance procedures. All personnel, contractors, and third parties with access to General Bots systems must understand and comply with this policy.


## 1. Information Security Policy

### 1.1 Purpose and Scope

This Information Security Policy applies to all users, systems, and data within the General Bots infrastructure. It establishes the standards for protecting confidential information, maintaining system integrity, and ensuring business continuity across all operations.

### 1.2 Information Classification

We classify information into categories to ensure proper protection and appropriate resource allocation. Unclassified information can be made public without implications for the company, including marketing materials and public documentation. Employee Confidential information encompasses personal employee data such as medical records, salary information, performance reviews, and contact details. Company Confidential information includes business-critical assets such as contracts, source code, business plans, passwords for critical IT systems, client contact records, financial accounts, and strategic plans. Client Confidential information covers client personally identifiable information, passwords to client systems, client business plans, new product information, and market-sensitive information.

### 1.3 Security Objectives

Our security framework aims to reduce the risk of IT problems through proactive measures and continuous monitoring. We plan for problems and establish procedures to deal with them effectively when they occur. Our systems are designed to keep working even when something goes wrong through redundancy and failover capabilities. We protect company, client, and employee data through encryption, access controls, and monitoring. We keep valuable company information such as plans and designs confidential through strict access controls. We meet our legal obligations under the General Data Protection Regulation and other applicable laws. We fulfill our professional obligations towards our clients and customers through transparent practices and reliable service.

### 1.4 Roles and Responsibilities

Rodrigo Rodriguez serves as the director with overall responsibility for IT security strategy and policy approval. Pragmatismo Data Center functions as the IT partner organization we use to help with planning and technical support. The Data Protection Officer advises on data protection laws and best practices, reporting directly to senior management. All employees are responsible for following security policies and reporting security incidents promptly. System administrators are responsible for implementing and maintaining security controls according to this policy. Department heads are responsible for ensuring their teams comply with security policies and complete required training.

### 1.5 Review Process

We review this policy annually, with the next review scheduled for the date indicated in the document control section. Questions, suggestions, or feedback should be directed to security@pragmatismo.com.br for consideration during the review process or for immediate clarification.


## 2. Access Control Policy

### 2.1 Access Management Principles

Our access management follows four core principles. The Least Privilege principle ensures users receive only the minimum access rights necessary to perform their job functions. The Need-to-Know principle restricts access to confidential information to those who require it for their specific duties. Separation of Duties divides critical functions among multiple people to prevent fraud and error. Regular Reviews conducted quarterly ensure access rights remain appropriate as roles and responsibilities evolve.

### 2.2 User Account Management

Account creation follows a controlled process where new accounts are created only upon approval from the user's manager. Default accounts are disabled immediately after system installation to prevent unauthorized access. Each user has a unique account because shared accounts are strictly prohibited to maintain accountability.

Account modification requires manager approval for any access changes. Privilege escalation requires security team approval in addition to manager approval. All changes are logged and reviewed monthly to detect anomalies.

Account termination procedures ensure accounts are disabled within 2 hours of employment termination. Access is revoked immediately for terminated employees without exception. Contractor accounts expire automatically at contract end. All company devices and access credentials must be returned before departure.

### 2.3 Access Review Procedures

Monthly reviews examine privileged account usage patterns, check for inactive accounts that have been dormant for more than 30 days, and verify that administrative access justifications remain valid.

Quarterly reviews require department heads to review all team member access, remove unnecessary permissions, and document review results along with any actions taken.

Annual reviews conduct a comprehensive examination of all user accounts, validate role-based access assignments against current organizational structure, and audit system administrator privileges for appropriateness.


## 3. Password Policy

### 3.1 Password Requirements

Password complexity requirements mandate a minimum of 12 characters for standard users and 16 characters for administrative accounts. Passwords must include uppercase letters, lowercase letters, numbers, and special characters. Passwords cannot contain the username or common dictionary words.

Password lifetime requirements specify 90-day rotation for standard accounts, 60-day rotation for administrative accounts, and 180-day rotation for service accounts with documented exceptions approved by the security team.

Password history settings ensure the system remembers the last 12 passwords, and users cannot reuse any of these previous passwords when setting a new one.

### 3.2 Password Storage and Transmission

All passwords are hashed using the Argon2id algorithm, which provides strong resistance against both CPU and GPU-based attacks. Passwords are never stored in plaintext under any circumstances. Passwords are never transmitted via email or unencrypted channels. Password managers are recommended for secure storage of credentials.

### 3.3 Multi-Factor Authentication

Multi-factor authentication is required for all administrative accounts, remote access connections, access to confidential data, and financial system access.

Acceptable MFA methods include Time-based One-Time Passwords (TOTP) as the preferred method, hardware tokens such as YubiKey, SMS codes only as a backup method due to SIM-swapping risks, and biometric authentication where available and appropriate.


## 4. Data Protection Policy

### 4.1 Data Encryption

Encryption at rest protects stored data across all systems. Databases use AES-256-GCM encryption for sensitive fields. File storage applies AES-256-GCM encryption to all uploaded files. Backups are encrypted before transmission and storage. Mobile devices require full-disk encryption.

Encryption in transit protects data during transmission. All external communications use TLS 1.3. Service-to-service communication uses mutual TLS (mTLS). Remote access requires VPN connections. Certificate pinning applies to critical services to prevent man-in-the-middle attacks.

### 4.2 Data Retention and Disposal

Retention periods define how long different data types are kept. User data is retained as long as the account is active plus 30 days after closure. Audit logs are retained for 7 years to meet compliance requirements. Full backups are retained for 90 days while incremental backups are retained for 30 days. Email is retained for 2 years unless a legal hold applies.

Secure disposal ensures data cannot be recovered after deletion. Digital data undergoes secure deletion with multiple overwrites. Physical media is destroyed through shredding or degaussing. Certificates of destruction are maintained for 3 years as proof of proper disposal.

### 4.3 Data Privacy and GDPR Compliance

We classify and process only information necessary for the completion of our duties. We limit access to personal data to only those who need it for processing. Our classification system ensures information is protected properly and that we allocate security resources appropriately based on sensitivity levels.

User rights under GDPR are fully supported. Users have the right to access their personal data upon request. Users have the right to correction of inaccurate data. Users have the right to deletion, also known as the right to be forgotten. Users have the right to data portability in machine-readable formats. Users have the right to restrict processing of their data.

Data breach notification follows strict timelines. Breach assessment must be completed within 24 hours of discovery. Notification to authorities occurs within 72 hours if required by regulation. User notification happens without undue delay when their data is affected. All breaches are documented regardless of whether notification is required.


## 5. Incident Response Plan

### 5.1 Incident Classification

Incidents are classified into four severity levels to guide response priorities and resource allocation.

Critical incidents (P1) include active data breaches with confirmed data exfiltration, ransomware infections affecting production systems, complete system outages affecting all users, and compromise of administrative credentials. These require immediate response with all available resources.

High priority incidents (P2) include suspected data breaches under investigation, malware infections on non-critical systems, unauthorized access attempts that were detected, and partial system outages affecting critical services.

Medium priority incidents (P3) include failed security controls requiring attention, policy violations without immediate risk, minor system vulnerabilities discovered, and isolated user account compromises.

Low priority incidents (P4) include security alerts requiring investigation, policy clarification needs, security awareness issues, and minor configuration issues.

### 5.2 Incident Response Procedures

Detection and reporting occurs within the first 0-15 minutes. Security incidents are detected via monitoring systems or reported by users. Initial assessment determines severity level. The incident is logged in the tracking system. The security team is notified immediately for P1 and P2 incidents, or within 1 hour for P3 and P4 incidents.

Containment occurs from 15 minutes to 2 hours after detection. Affected systems are isolated from the network. Compromised accounts are disabled. Evidence is preserved for investigation. Temporary security controls are implemented. Management and stakeholders are notified.

Investigation occurs from 2 to 24 hours after containment. Logs and forensic evidence are gathered. Attack vectors and scope are analyzed. Root cause is identified. Findings are documented. A determination is made whether external authorities need notification.

Eradication typically takes 1-3 days. Malware and unauthorized access are removed. Vulnerabilities are patched. Compromised credentials are reset. Additional security controls are applied. Systems are verified to be clean.

Recovery typically takes 1-5 days. Systems are restored from clean backups if needed. Systems gradually return to production. Enhanced monitoring watches for re-infection. System functionality is validated. User communication and support is provided.

Post-incident review occurs within 1 week. The complete incident timeline is documented. Response effectiveness is analyzed. Lessons learned are identified. Security controls are updated. Detection capabilities are improved. Incident response procedures are updated based on findings.

### 5.3 Contact Information

Internal contacts for security matters include the Security Team at security@pragmatismo.com.br, IT Support at support@pragmatismo.com.br, and Management through Rodrigo Rodriguez.

External contacts should be maintained in a separate secure document and include local law enforcement authorities, legal counsel, the relevant Data Protection Authority, and the cyber insurance provider.

### 5.4 Communication Plan

Internal communication follows escalation timelines. The security team and management are notified immediately. Affected department heads are notified within 2 hours. All staff are notified within 4 hours if the impact is widespread. Daily updates continue during active incidents.

External communication follows regulatory requirements. Customers are notified within 24 hours if their data is affected. Partners are notified within 12 hours if systems are shared. Authorities are notified within 72 hours per GDPR requirements. Public and media communication occurs only through the designated spokesperson.


## 6. Backup and Recovery Procedures

### 6.1 Backup Schedule

Full backups run weekly on Sundays at 2:00 AM and include all databases, file storage, and configurations. Full backups are retained for 12 weeks and stored in a geographically separate location.

Incremental backups run daily at 2:00 AM and include only changed files and database transactions since the last backup. Incremental backups are retained for 30 days and stored both locally and replicated off-site.

Continuous backups capture database transaction logs every 15 minutes and critical configuration changes immediately. These are retained for 7 days and enable point-in-time recovery to any moment within that window.

### 6.2 Backup Verification

Automated testing runs continuously. Daily tests verify backup completion. Weekly tests restore sample files. Monthly tests perform full database restoration to an isolated environment.

Manual testing occurs on a scheduled basis. Quarterly tests conduct full disaster recovery drills. Bi-annual tests perform complete system restoration to an alternate site. Annual tests execute a full business continuity exercise with stakeholders.

### 6.3 Recovery Procedures

Recovery Time Objectives (RTO) define maximum acceptable downtime. Critical systems must recover within 4 hours. Important systems must recover within 24 hours. Non-critical systems must recover within 72 hours.

Recovery Point Objectives (RPO) define maximum acceptable data loss. Critical data has an RPO of 15 minutes. Important data has an RPO of 24 hours. Non-critical data has an RPO of 1 week.

Recovery steps follow a systematic process. First, assess damage and determine recovery scope. Second, verify backup integrity before beginning restoration. Third, restore to an isolated environment first for validation. Fourth, validate data integrity and completeness. Fifth, test system functionality thoroughly. Sixth, switch users to recovered systems. Seventh, monitor for issues during the transition period. Eighth, document the recovery process and timing for future reference.


## 7. Change Management Procedures

### 7.1 Change Categories

Standard changes are pre-approved routine modifications. These include security patches applied within 48 hours of release and user account modifications. Standard changes require only manager sign-off without additional approval.

Normal changes are non-emergency modifications requiring testing. These include software updates, new features, and infrastructure modifications. Normal changes require Change Advisory Board approval before implementation.

Emergency changes address critical security issues or outages. These include critical security patches, system outage fixes, and active threat mitigation. Emergency changes receive expedited approval from the Security Director.

### 7.2 Change Request Process

The change process follows eight steps. Submission requires completing the change request form with full details. Risk assessment evaluates potential security impact. Approval is obtained based on change type requirements. Testing validates the change in a non-production environment. Scheduling places the change during an appropriate maintenance window. Implementation executes the change with a rollback plan ready. Verification confirms the change was successful. Documentation updates configuration records to reflect the change.

### 7.3 Change Testing Requirements

Test cases must cover functionality validation, security control verification, performance impact assessment, user acceptance testing, and rollback procedure verification.

Test environments progress through stages. Development supports individual developer testing. Staging handles integration and security testing. Pre-production hosts user acceptance testing. Production uses phased rollout with enhanced monitoring.


## 8. Security Incident Procedures

### 8.1 Reporting Security Incidents

Incidents can be reported through several channels. Email reports go to security@pragmatismo.com.br. Phone reports use the security hotline. Web reports use the internal incident reporting portal. In-person reports can be made directly to the IT department.

Reportable events include suspicious emails or phishing attempts, lost or stolen devices, unauthorized access or unusual system behavior, malware alerts, data leaks or exposures, policy violations, and any security concerns or vulnerabilities discovered.

Timing requirements specify immediate reporting for critical incidents, reporting within 1 hour for high-priority incidents, and same business day reporting for medium and low priority incidents.

### 8.2 Employee Response to Incidents

When an incident occurs, employees should report immediately to the security team, preserve evidence by not deleting suspicious emails, disconnect their device from the network if it may be compromised, document what happened while details are fresh, and follow instructions from the security team.

Employees should avoid trying to fix the problem themselves, deleting or modifying potential evidence, discussing the incident on social media, blaming others, or ignoring suspicious activity hoping it will resolve itself.


## 9. Data Breach Response Procedures

### 9.1 Immediate Response

Within the first 24 hours, the response team must contain the breach to stop ongoing data exposure, assess the situation to determine scope and data affected, notify the security team and management, preserve logs and forensic data as evidence, and begin documenting the incident timeline.

### 9.2 Investigation Phase

During the 1-3 day investigation phase, forensic specialists conduct detailed analysis of the breach. The scope determination identifies all affected systems and data. Root cause analysis determines how the breach occurred. Impact analysis assesses damage and ongoing risks. Legal review consults with the legal team on notification obligations.

### 9.3 Notification Requirements

Internal notification follows escalation timelines. Management is notified immediately. Legal is notified within 2 hours. PR and Communications are notified within 4 hours. Affected departments are notified within 8 hours.

External notification follows regulatory requirements. Data Protection Authorities must be notified within 72 hours per GDPR requirements. Affected individuals must be notified without undue delay. Business partners must be notified within 24 hours if their data is affected. Law enforcement is notified as required by jurisdiction.

### 9.4 Remediation and Prevention

Following a breach, the organization applies security patches and fixes to close vulnerabilities. Compromised credentials are reset across all affected systems. Monitoring and detection capabilities are enhanced to catch similar attacks. Security controls are updated based on lessons learned. Additional security training is provided to affected teams. Policies are reviewed and updated to address gaps. All lessons learned are implemented to prevent recurrence.


## 10. Regular Maintenance Tasks

### 10.1 Weekly Tasks

Security updates are reviewed and critical security patches are applied. Antivirus and antimalware signatures are updated. Security alerts and events are reviewed. Backup completion status is checked. System resource usage is monitored for anomalies.

Automated processes run continuously including vulnerability scans, log analysis and correlation, backup integrity checks, and certificate expiration monitoring.

### 10.2 Monthly Tasks

Access reviews examine new user accounts created during the month, audit privileged account usage, check for inactive accounts dormant for more than 30 days, review failed login attempts for patterns, and validate group memberships remain appropriate.

System maintenance applies non-critical patches, reviews system performance metrics, updates system documentation, tests disaster recovery procedures, and reviews incident reports from the month.

### 10.3 Quarterly Tasks

Compliance audits review security policy compliance, audit access controls and permissions, verify encryption implementations, check backup and recovery processes, and validate security configurations against baselines.

Security assessments conduct internal vulnerability assessments, run phishing simulation exercises, deliver security awareness training, review third-party security posture, and update risk assessments.

### 10.4 Annual Tasks

Penetration testing engages a certified firm for external penetration testing, conducts internal network penetration testing, performs application security testing, executes social engineering assessments, and remediates all findings within 90 days.

Disaster recovery testing conducts a full disaster recovery drill, tests alternate site failover, executes a business continuity exercise, updates recovery procedures based on results, and documents lessons learned.

Policy and documentation work includes annual policy review and updates, security training for all staff, updating security documentation, reviewing vendor security agreements, and strategic security planning for the coming year.

### 10.5 Bi-Annual Tasks

Disaster recovery testing at the semi-annual level includes complete system restoration to an alternate site, database recovery to a specific point-in-time, application functionality verification, network failover testing, and communication system testing.

Business continuity testing includes testing emergency communication procedures, verifying contact information is current, reviewing and updating the business continuity plan, testing backup data center capabilities, and validating recovery time objectives are achievable.


## 11. Employees Joining and Leaving

We provide comprehensive training to new staff and ongoing support for existing staff to implement this policy. Initial training covers an introduction to IT security including risks, basic security measures, company policies, and where to get help. Each employee completes appropriate security awareness training. Training covers how to use company systems and security software properly. Staff can request a security health check on their computer, tablet, or phone. Access to systems and resources is granted based on job role requirements. Appropriate security tools are assigned including VPN access, password manager, and MFA devices.

The onboarding security checklist ensures all steps are completed. Background checks are completed where applicable. The security policy acknowledgment is signed. Security training is completed. NDA and confidentiality agreements are signed. User accounts are created with appropriate permissions. MFA is configured for all accounts. Company devices are issued and configured. VPN access is configured if needed. A password manager account is created. Emergency contact information is collected.

When people leave a project or the company, we promptly revoke their access privileges to all systems.

The offboarding security checklist ensures thorough access removal. All user accounts are disabled within 2 hours of departure. VPN and remote access are revoked. The former employee is removed from all groups and distribution lists. Company devices including laptops, phones, and tokens are collected. Access cards and keys are collected. Any shared account passwords the person knew are reset. The person is removed from third-party systems such as GitHub and AWS. Ownership of documents and files is transferred. An exit interview covers ongoing security obligations. Documentation confirms all access revocation is completed.


## 12. Data Protection Officer Responsibilities

The company ensures the Data Protection Officer is given all appropriate resources to carry out their tasks and maintain their expert knowledge. The DPO reports directly to the highest level of management and must not carry out any other tasks that could result in a conflict of interest.

The DPO's duties include monitoring compliance with GDPR and other privacy regulations, advising on data protection impact assessments, cooperating with supervisory authorities, acting as the contact point for data subjects exercising their rights, maintaining records of processing activities, providing data protection training to staff, conducting privacy audits, and reviewing privacy policies and procedures for adequacy.


## 13. Technical Documentation Requirements

### 13.1 Network Architecture Documentation

Required network documentation includes network topology diagrams showing both logical and physical layouts, IP address allocation schemes, firewall rules and security zone definitions, VPN configurations, DMZ architecture, network device inventory, VLAN configurations, and routing protocols and tables.

This documentation must be updated within 48 hours of any network change to remain accurate.

### 13.2 System Configuration Documentation

Required system documentation includes server inventory with roles and specifications, operating system versions and patch levels, installed software and versions, service configurations, database schemas and configurations, application architecture diagrams, API documentation, and integration points and dependencies.

This documentation must be updated within 24 hours of configuration changes.

### 13.3 Security Controls Documentation

Security control documentation covers access control lists, security group configurations, intrusion detection and prevention rules, data loss prevention policies, endpoint protection configurations, email security settings, web filtering rules, and security monitoring dashboards.

This documentation is reviewed monthly with a comprehensive review conducted quarterly.

### 13.4 Encryption Standards Documentation

Encryption documentation specifies encryption algorithms in use such as AES-256-GCM and TLS 1.3, key management procedures, certificate inventory and renewal schedule, data classification and encryption requirements, encryption at rest implementations, encryption in transit configurations, and cryptographic library versions.

This documentation must be updated immediately upon any encryption-related change.

### 13.5 Logging and Monitoring Documentation

Logging documentation covers log sources and types collected, log retention periods, log storage locations and capacity, log analysis tools and procedures, alert thresholds and escalation paths, monitoring dashboards and reports, and SIEM configuration and rules.

This documentation is reviewed quarterly with an annual comprehensive audit.


## 14. Compliance Records Management

### 14.1 Risk Assessment Reports

Risk assessments are conducted annually for comprehensive organizational assessment, quarterly for targeted assessments of new systems and services, and ad-hoc after significant incidents or changes.

Risk assessment reports contain identified assets and their value to the organization, threat identification and analysis, vulnerability assessment, risk likelihood and impact ratings, risk treatment plans, residual risk acceptance decisions, and review and approval signatures.

Risk assessment records are retained for 7 years.

### 14.2 Audit Logs

Log types collected include authentication and authorization events, administrative actions, data access operations including reads, writes, and deletes, configuration changes, security events and alerts, system errors and failures, and network traffic logs.

Retention periods vary by log type. Security logs are retained for 7 years. System logs are retained for 1 year. Application logs are retained for 90 days. Network logs are retained for 30 days.

Log protection requirements specify that logs are read-only after creation, encrypted in transit and at rest, backed up daily, and monitored for tampering.

### 14.3 Training Records

Training requirements include new hire security orientation within the first week of employment, annual security awareness training for all staff, role-specific security training as applicable to job function, phishing simulation exercises quarterly, and incident response training for the security team annually.

Training documentation includes training completion dates, training content and version delivered, assessment scores if applicable, certificates of completion, and refresher training schedules.

Training records are retained for the duration of employment plus 3 years.

### 14.4 Incident Reports

Incident reports must include the detection date and time, incident classification and severity, systems and data affected, timeline of events, response actions taken, root cause analysis, lessons learned, and corrective actions implemented.

Reports are distributed internally to management, the security team, and affected departments. External distribution follows regulatory and contractual requirements.

Incident reports are retained for 7 years.

### 14.5 Access Review Records

Review documentation includes the date of review, reviewer name and title, list of accounts reviewed, access changes made, justification for access granted, exceptions and approvals, and follow-up actions required.

Review schedules specify quarterly reviews for standard users, monthly reviews for privileged users, and bi-annual reviews for service accounts.

Access review records are retained for 3 years.


## 15. Compliance Framework

### 15.1 Applicable Regulations

GDPR compliance requires data protection impact assessments for high-risk processing, privacy by design and by default in all systems, user consent management, data subject rights fulfillment, and breach notification procedures.

SOC 2 compliance requires security controls documentation, availability monitoring, confidentiality protection measures, privacy practices documentation, and annual audit compliance verification.

ISO 27001 compliance requires an information security management system, risk assessment and treatment processes, security controls implementation, continuous improvement processes, and regular internal audits.

### 15.2 Compliance Monitoring

Automated monitoring tracks security control effectiveness, policy compliance through scanning, configuration drift detection, vulnerability management status, and patch compliance levels.

Manual reviews include quarterly compliance assessments, annual third-party audits, internal audit programs, management review meetings, and regulatory requirement updates.


## 16. Third-Party Security

### 16.1 Vendor Security Assessment

Pre-contract assessment requires security questionnaire completion, security certification review for SOC 2 and ISO 27001, data processing agreement execution, security requirements in the contract, and incident notification requirements.

Ongoing monitoring includes annual security re-assessment, review of security incidents involving the vendor, audit report review, performance measurement against SLAs, and security scorecard maintenance.

### 16.2 Data Sharing with Third Parties

Data sharing requirements include having a data processing agreement in place, sharing only the minimum necessary data, encryption for all data in transit, access controls and monitoring, and the right to audit vendor security practices.

The approval process requires security team review, legal review of agreements, privacy impact assessment, management approval for sensitive data sharing, and documentation in the vendor register.


## 17. Vulnerability Management

### 17.1 Vulnerability Identification

Vulnerabilities are identified through multiple sources including automated vulnerability scanning conducted weekly, annual penetration testing by external firms, security research and advisories from vendors and researchers, bug bounty program submissions, internal security testing, and third-party security assessments.

### 17.2 Vulnerability Remediation

Response times are based on severity. Critical vulnerabilities must be remediated within 24 hours. High severity vulnerabilities must be remediated within 7 days. Medium severity vulnerabilities must be remediated within 30 days. Low severity vulnerabilities must be remediated within 90 days or formally accepted as risk.

The remediation process follows a structured approach. First, the vulnerability is confirmed and documented. Second, impact and exploitability are assessed. Third, a remediation plan is developed. Fourth, the patch or fix is tested in non-production. Fifth, the change management process is followed. Sixth, the fix is deployed to production. Seventh, verification testing confirms the fix is effective. Eighth, documentation is updated.

### 17.3 Reporting a Vulnerability

External security researchers can report vulnerabilities by email to security@pragmatismo.com.br. A PGP key is available on the website for encrypted communication. Initial response is provided within 48 hours. A bug bounty program provides rewards for qualifying vulnerabilities.

Internal staff should report vulnerabilities via the internal security portal or email the security team directly for critical issues. Reports should include a description of the vulnerability, affected systems, and steps to reproduce the issue. Response is provided within 24 hours.


## 18. Security Metrics and KPIs

### 18.1 Key Performance Indicators

Security metrics track operational effectiveness. Mean time to detect (MTTD) incidents has a target of less than 15 minutes. Mean time to respond (MTTR) to incidents has a target of less than 4 hours. Percentage of systems with latest patches has a target of greater than 95%. Failed login attempts per day are baselined at less than 100. Security training completion rate has a target of 100%. Vulnerabilities remediated within SLA has a target of greater than 90%. Backup success rate has a target of 100%. Access review completion has a target of 100% on schedule.

Reporting occurs at multiple intervals. Weekly reports cover security incidents and critical metrics. Monthly reports provide a comprehensive security dashboard. Quarterly reports analyze metrics trends. Annual reports assess overall security posture.


## 19. Policy Enforcement

### 19.1 Policy Violations

Types of violations include unauthorized access attempts, password sharing, installation of unauthorized software, data exfiltration or leakage, policy non-compliance, and failure to report incidents.

Consequences follow progressive discipline. First offense results in a warning and mandatory retraining. Second offense results in a written warning and management review. Third offense results in suspension or termination. Severe violations result in immediate termination and potential legal action.

### 19.2 Exception Process

Exception requests require written justification, a completed risk assessment, identification of compensating controls, time-limited approval with a maximum of 90 days, approval from both management and the security team, and regular review while the exception remains active.


## 20. Document Control

This document is owned by Rodrigo Rodriguez, Security Director. The last update date and next review date are indicated in the document header. The current version is 2.0 with approved status.

The change history shows Version 1.0 as the initial policy creation and Version 2.0 as the comprehensive expansion with detailed procedures.

Distribution includes all employees via the internal portal, availability to clients upon request, and a summary published on the company website.

Approval authority, approval date, and next review date are recorded in the document management system.


## Contact Information

The Security Team can be reached by email at security@pragmatismo.com.br, by phone at the emergency hotline maintained in internal systems, or through the internal security portal.

Specific inquiries should be directed to appropriate addresses. Security incidents go to security@pragmatismo.com.br. Privacy concerns go to privacy@pragmatismo.com.br. Compliance questions go to compliance@pragmatismo.com.br. General IT support requests go to support@pragmatismo.com.br.