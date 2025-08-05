# 5. Process Mapping

## Key Workflows
- User onboarding and account setup
- Problem capture and organization workflow
- Study session and performance tracking
- Cloud synchronization for premium users
- Problem review and revision workflow

## User Onboarding Process
```mermaid
flowchart TD
    Start([User Downloads App]) --> Install[Install Desktop App]
    Install --> Launch[First Launch]
    Launch --> Setup[Initial Setup Wizard]
    Setup --> CreateAccount{Create Account?}
    
    CreateAccount -->|Yes| Register[Registration Form]
    CreateAccount -->|No| LocalMode[Continue with Local Storage]
    
    Register --> EmailVerify[Email Verification]
    EmailVerify --> Premium{Choose Premium?}
    
    Premium -->|Yes| Stripe[Stripe Payment Setup]
    Premium -->|No| FreeAccount[Free Account Created]
    
    Stripe --> PremiumAccount[Premium Account Created]
    
    LocalMode --> OnboardingTour[App Tour & Tutorial]
    FreeAccount --> OnboardingTour
    PremiumAccount --> OnboardingTour
    
    OnboardingTour --> FirstScreenshot[Take First Screenshot]
    FirstScreenshot --> Ready([Ready to Use])
```

## Problem Capture and Organization Workflow
```mermaid
flowchart TD
    Start([User Encounters Problem]) --> Capture{How to Capture?}
    
    Capture -->|Desktop| Screenshot[Press Keybind for Screenshot]
    Capture -->|Web| Upload[Upload Image File]
    
    Screenshot --> Overlay[Overlay UI Appears]
    Upload --> Form[Fill Problem Form]
    
    Overlay --> FillDetails[Fill Problem Details]
    FillDetails --> SelectOrg[Select/Create Organization]
    Form --> SelectOrg
    
    SelectOrg --> ChooseFolder[Choose/Create Folder]
    ChooseFolder --> ChooseCourse[Choose/Create Course]
    ChooseCourse --> ChooseSubject[Choose/Create Subject]
    
    ChooseSubject --> Performance[Add Performance Data]
    Performance --> TimeSpent[Time Spent]
    TimeSpent --> Difficulty[Difficulty Rating]
    Difficulty --> Confidence[Confidence Level]
    Confidence --> Notes[Optional Notes]
    
    Notes --> Save[Save Problem]
    Save --> LocalSave[Save to Local Storage]
    
    LocalSave --> Premium{Premium User?}
    Premium -->|Yes| CloudSync[Background Cloud Sync]
    Premium -->|No| Complete([Problem Saved])
    
    CloudSync --> Complete
```

## Study Session and Performance Tracking
```mermaid
flowchart TD
    Start([Begin Study Session]) --> Mode{Study Mode?}
    
    Mode -->|Browse| Browse[Browse Problems by Organization]
    Mode -->|Revision| Revision[Enter Revision Mode]
    Mode -->|Search| Search[Search Problems]
    
    Browse --> SelectProblem[Select Problem to Review]
    Revision --> FilterScope[Select Scope & Performance Filter]
    Search --> SearchResults[View Search Results]
    
    FilterScope --> RevisionList[Show Filtered Problems]
    SearchResults --> SelectProblem
    RevisionList --> SelectProblem
    
    SelectProblem --> ViewProblem[Display Problem & Image]
    ViewProblem --> Attempt{Attempt Problem?}
    
    Attempt -->|Yes| StartTimer[Start Timer]
    Attempt -->|No| NextProblem[Go to Next Problem]
    
    StartTimer --> Solve[Work on Problem]
    Solve --> Submit[Submit/Mark Complete]
    Submit --> RateAttempt[Rate Difficulty & Confidence]
    RateAttempt --> RecordAttempt[Record Attempt Data]
    
    RecordAttempt --> UpdateStats[Update Problem Statistics]
    UpdateStats --> NextProblem
    
    NextProblem --> Mode
```

## Cloud Synchronization Process (Premium)
```mermaid
flowchart TD
    Start([Sync Trigger]) --> TriggerType{Trigger Type?}
    
    TriggerType -->|App Start| AppStart[App Startup Sync]
    TriggerType -->|Timer| Scheduled[Scheduled Background Sync]
    TriggerType -->|Manual| Manual[User-Initiated Sync]
    
    AppStart --> CheckAuth[Check Authentication]
    Scheduled --> CheckAuth
    Manual --> CheckAuth
    
    CheckAuth --> AuthValid{Auth Valid?}
    AuthValid -->|No| ReAuth[Re-authenticate User]
    AuthValid -->|Yes| StartSync[Begin Sync Process]
    
    ReAuth --> Login[Login Flow]
    Login --> StartSync
    
    StartSync --> Push[Push Local Changes]
    Push --> GetLocalChanges[Get Unsynced Local Data]
    GetLocalChanges --> UploadChanges[Upload to Cloud]
    UploadChanges --> MarkSynced[Mark as Synced]
    
    MarkSynced --> Pull[Pull Cloud Changes]
    Pull --> GetCloudChanges[Get Remote Updates]
    GetCloudChanges --> ConflictCheck{Conflicts Detected?}
    
    ConflictCheck -->|Yes| ResolveConflicts[Conflict Resolution]
    ConflictCheck -->|No| ApplyChanges[Apply Cloud Changes]
    
    ResolveConflicts --> ApplyChanges
    ApplyChanges --> UpdateLocal[Update Local Storage]
    UpdateLocal --> SyncComplete([Sync Complete])
```

## Problem Review and Revision Workflow
```mermaid
flowchart TD
    Start([Enter Revision Mode]) --> SelectCriteria[Select Review Criteria]
    
    SelectCriteria --> Scope[Choose Scope]
    Scope --> Performance[Filter by Performance]
    Performance --> TimeFrame[Set Time Frame]
    
    TimeFrame --> GenerateSet[Generate Problem Set]
    GenerateSet --> DisplayProblems[Display Filtered Problems]
    
    DisplayProblems --> ReviewLoop{More Problems?}
    
    ReviewLoop -->|Yes| NextProblem[Select Next Problem]
    ReviewLoop -->|No| SessionComplete[Review Session Complete]
    
    NextProblem --> ShowProblem[Display Problem]
    ShowProblem --> UserAction{User Action?}
    
    UserAction -->|Attempt Again| RecordAttempt[Record New Attempt]
    UserAction -->|Mark Mastered| UpdateStatus[Update Mastery Status]
    UserAction -->|Edit Problem| EditProblem[Edit Problem Details]
    UserAction -->|Skip| SkipProblem[Skip to Next]
    
    RecordAttempt --> UpdatePerformance[Update Performance Metrics]
    UpdateStatus --> UpdatePerformance
    EditProblem --> UpdatePerformance
    SkipProblem --> ReviewLoop
    
    UpdatePerformance --> ReviewLoop
    
    SessionComplete --> ShowSummary[Show Session Summary]
    ShowSummary --> SaveProgress[Save Session Progress]
    SaveProgress --> End([End Session])
```
