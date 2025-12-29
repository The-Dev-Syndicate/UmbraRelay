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
}

export interface Source {
  id: number;
  type: 'rss' | 'github';
  name: string;
  config_json: Record<string, any>;
  enabled: boolean;
  last_synced_at: number | null;
}

export interface SourceInput {
  source_type: string;
  name: string;
  config_json: Record<string, any>;
  token?: string;
}

export interface UpdateSourceInput {
  name?: string;
  config_json?: Record<string, any>;
  enabled?: boolean;
  token?: string;
}

