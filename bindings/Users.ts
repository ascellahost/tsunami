export interface Users {
  discord_id: string;
  domain: string;
  id: number;
  key: string;
  name: string;
  autodelete: number | null;
  deleteall: number | null;
  upload_key: string | null;
  url_style: number;
  invite_code: string | null;
  invited_by: number;
  lang: string;
  flags: number;
}
