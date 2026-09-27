import { parse_ndjson, type NewlineDeliminatedJson } from './ndjson';
import { req } from './utils.svelte';

export type AnalysisReport = {
    metadata: ReportMetadata;
    rows: AnalysisRow[];
    statistics: ReportStatistics;
};

export type ReportStatistics = {
    num_warnings: number;
    num_informational_logs: number;
    num_skipped_packets: number;
};

export class ReportMetadata {
    public analyzers: AnalyzerMetadata[];
    public rayhunter: RayhunterMetadata;
    public report_version: number;

    constructor(ndjson: any) {
        this.analyzers = ndjson.analyzers;
        this.rayhunter = ndjson.rayhunter;
        this.report_version = ndjson.report_version || 2; // Default to v2
    }
}

export type RayhunterMetadata = {
    rayhunter_version: string;
    system_os: string;
    arch: string;
};

export type AnalyzerMetadata = {
    name: string;
    description: string;
    version: number;
};

export type AnalysisRow = SkippedPacket | PacketAnalysis;
export enum AnalysisRowType {
    Skipped,
    Analysis,
}

export type SkippedPacket = {
    type: AnalysisRowType.Skipped;
    reason: string;
};

export type PacketAnalysis = {
    type: AnalysisRowType.Analysis;
    packet_timestamp: Date;
    events: Event[];
};

export type EventType = 'Informational' | 'Low' | 'Medium' | 'High';

export type Event = {
    event_type: EventType;
    message: string;
    analyzer_index: number;
};

function parse_event_type(raw: string): EventType {
    if (!['Informational', 'Low', 'Medium', 'High'].includes(raw)) {
        throw `Invalid/unhandled event type: ${raw}`;
    }
    return raw as EventType;
}

function get_rows(row_jsons: any[]): AnalysisRow[] {
    const rows: AnalysisRow[] = [];
    for (const row_json of row_jsons) {
        if (row_json.skipped_message_reason) {
            rows.push({
                type: AnalysisRowType.Skipped,
                reason: row_json.skipped_message_reason,
            });
        }
        const raw_events: any[] = row_json.events ?? [];
        // Detect V3 (dense, each element has analyzer_index) vs V2 (sparse, nulls allowed).
        const first_non_null = raw_events.find((e) => e !== null);
        const is_v3 = first_non_null !== undefined && first_non_null.analyzer_index !== undefined;
        const events: Event[] = is_v3
            ? raw_events.map((e: any) => ({
                  event_type: parse_event_type(e.event_type),
                  message: e.message,
                  analyzer_index: e.analyzer_index,
              }))
            : raw_events
                  .map((e: any, i: number): Event | null => {
                      if (e === null) {
                          return null;
                      }
                      return {
                          event_type: parse_event_type(e.event_type),
                          message: e.message,
                          analyzer_index: i,
                      };
                  })
                  .filter((e): e is Event => e !== null);
        if (events.length > 0) {
            rows.push({
                type: AnalysisRowType.Analysis,
                packet_timestamp: new Date(row_json.packet_timestamp),
                events,
            });
        }
    }
    return rows;
}

function get_report_stats(rows: AnalysisRow[]): ReportStatistics {
    let num_warnings = 0;
    let num_informational_logs = 0;
    let num_skipped_packets = 0;
    for (const row of rows) {
        if (row.type === AnalysisRowType.Skipped) {
            num_skipped_packets++;
        } else {
            for (const event of row.events) {
                if (event.event_type === 'Informational') {
                    num_informational_logs++;
                } else {
                    num_warnings++;
                }
            }
        }
    }
    return {
        num_warnings,
        num_informational_logs,
        num_skipped_packets,
    };
}

export function parse_finished_report(report_json: NewlineDeliminatedJson): AnalysisReport {
    const metadata = new ReportMetadata(report_json[0]);
    const rows = get_rows(report_json.slice(1));
    const statistics = get_report_stats(rows);
    return {
        statistics,
        metadata,
        rows,
    };
}

export async function get_report(name: string): Promise<AnalysisReport> {
    const report_json = parse_ndjson(await req('GET', `/api/analysis-report/${name}`));
    return parse_finished_report(report_json);
}
