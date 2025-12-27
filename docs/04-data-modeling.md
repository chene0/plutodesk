# 4. Data Modeling

## Entities & Relationships
- **Users**: Account information and subscription status
- **Folders**: Top-level organization (e.g., "Fall 2024", "Personal Study")
- **Courses**: Subject areas within folders (e.g., "Calculus I", "Physics 101")
- **Subjects**: Topics within courses (e.g., "Derivatives", "Kinematics")
- **Problems**: Individual problem instances with metadata and performance data
- **Problem Attempts**: Individual solve attempts with performance metrics
- **Subscriptions**: Premium subscription management via Stripe

## ER Diagram
```mermaid
erDiagram
    USER ||--o{ FOLDER : owns
    USER ||--o{ SUBSCRIPTION : has
    FOLDER ||--o{ COURSE : contains
    COURSE ||--o{ SUBJECT : contains
    SUBJECT ||--o{ PROBLEM : contains
    PROBLEM ||--o{ PROBLEM_ATTEMPT : has
    
    USER {
        uuid id PK
        string email UK
        string password_hash
        string name
        timestamp created_at
        timestamp updated_at
        boolean is_premium
        timestamp last_sync
    }
    
    SUBSCRIPTION {
        uuid id PK
        uuid user_id FK
        string stripe_customer_id
        string stripe_subscription_id
        string status
        timestamp current_period_start
        timestamp current_period_end
        timestamp created_at
    }
    
    FOLDER {
        uuid id PK
        uuid user_id FK
        string name
        string description
        integer sort_order
        timestamp created_at
        timestamp updated_at
        boolean is_synced
    }
    
    COURSE {
        uuid id PK
        uuid folder_id FK
        string name
        string description
        string color_code
        integer sort_order
        timestamp created_at
        timestamp updated_at
        boolean is_synced
    }
    
    SUBJECT {
        uuid id PK
        uuid course_id FK
        string name
        string description
        integer sort_order
        timestamp created_at
        timestamp updated_at
        boolean is_synced
    }
    
    PROBLEM {
        uuid id PK
        uuid subject_id FK
        string title
        text description
        string image_path
        string s3_image_key
        integer confidence_level
        text notes
        timestamp created_at
        timestamp updated_at
        timestamp last_attempted
        integer attempt_count
        float success_rate
        boolean is_synced
        timestamp last_modified
    }
    
    PROBLEM_ATTEMPT {
        uuid id PK
        uuid problem_id FK
        integer time_spent_seconds
        integer difficulty_rating
        integer confidence_level
        boolean was_successful
        text notes
        timestamp attempted_at
        boolean is_synced
    }
```
