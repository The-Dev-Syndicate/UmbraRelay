export interface Item {
  id: number;
  source_id: number;
  external_id: string;
  title: string;
  summary: string | null;
  url: string;
  item_type: 'rss' | 'atom' | 'issue' | 'pr';
  state: 'unread' | 'read' | 'archived';
  created_at: number;
  updated_at: number;
  image_url?: string | null;
  content_html?: string | null;
  author?: string | null;
  category?: string | null; // JSON array string
  comments?: string | null;
  source_name?: string | null;
  source_group?: string | null;
}

export interface CustomView {
  id: number;
  name: string;
  source_ids?: string | null; // JSON array string
  group_names?: string | null; // JSON array string
  created_at: number;
  updated_at: number;
}

export interface CustomViewInput {
  name: string;
  sourceIds?: number[] | null;
  groupNames?: string[] | null;
}

export interface Group {
  id: number;
  name: string;
  created_at: number;
  updated_at: number;
}

export interface Source {
  id: number;
  source_type: 'rss' | 'atom' | 'github';
  name: string;
  config_json: Record<string, any> | string; // Can be string from backend or parsed object
  enabled: boolean;
  last_synced_at: number | null;
  group_ids?: number[]; // Group relationships via source_groups junction table
}

export interface Secret {
  id: number;
  name: string;
  ttl_type: 'relative' | 'absolute' | 'forever';
  ttl_value?: string | null;
  expires_at?: number | null;
  created_at: number;
  updated_at: number;
}

export interface GitHubRepository {
  id: number;
  name: string;
  full_name: string;
  private: boolean;
  html_url: string;
}

export interface SourceInput {
  source_type: string;
  name: string;
  config_json: Record<string, any>;
  token?: string; // Deprecated, use secret_id
  secret_id?: number;
  group_ids?: number[] | null;
}

export interface UpdateSourceInput {
  name?: string;
  config_json?: Record<string, any>;
  enabled?: boolean;
  token?: string; // Deprecated, use secret_id
  secret_id?: number | null; // None = don't update, null = clear, number = set
  group_ids?: number[]; // None = don't update, [] = clear groups, [1,2] = set groups
}

