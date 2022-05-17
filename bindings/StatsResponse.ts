import type { DisplayEmbed } from "./DisplayEmbed";

export interface StatsResponse {
  user_name: string;
  user_id: number;
  id: number;
  views: number;
  redirect?: string;
  content_type: string;
  image_size?: string;
  embed?: DisplayEmbed;
}
