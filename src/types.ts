export interface Item {
  id: number;
  source_id: number;
  external_id: string;
  title: string;
  summary: string | null;
  url: string;
  item_type: 'post' | 'issue' | 'pr';
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

export interface Source {
  id: number;
  source_type: 'rss' | 'github';
  name: string;
  config_json: Record<string, any> | string; // Can be string from backend or parsed object
  enabled: boolean;
  last_synced_at: number | null;
  group?: string | null;
}

export interface SourceInput {
  source_type: string;
  name: string;
  config_json: Record<string, any>;
  token?: string;
  group?: string | null;
}

export interface UpdateSourceInput {
  name?: string;
  config_json?: Record<string, any>;
  enabled?: boolean;
  token?: string;
  group?: string | null;
}

