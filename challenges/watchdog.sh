#!/bin/bash

# --- Configuration ---
# How long (in seconds) the CPU must be high before restarting
RESTART_AFTER_SECONDS=150 # 5 minutes

# How often (in seconds) to check the CPU usage
CHECK_INTERVAL=30
# ---------------------

# Declare an associative array to store state in memory.
# Keys will be container IDs, values will be Unix timestamps.
declare -A high_cpu_start_times

echo "Starting CPU monitor... Script must remain running."
echo "Will force-restart (kill) any compose service at >=99% CPU for $RESTART_AFTER_SECONDS seconds."

while true; do
    # Get all running container IDs for the current compose project
    # We also build a "set" of current containers for cleanup later
    declare -A current_running_containers
    CONTAINER_IDS=$(docker compose ps -q)

    if [ -z "$CONTAINER_IDS" ]; then
        # If no containers are running, clear our tracking array
        if [ ${#high_cpu_start_times[@]} -gt 0 ]; then
             echo "No containers running. Clearing all tracking."
             high_cpu_start_times=()
        fi
        sleep "$CHECK_INTERVAL"
        continue
    fi

    # Add all running container IDs to our "set"
    for ID in $CONTAINER_IDS; do
        current_running_containers[$ID]=1
    done

    # 1. Check CPU for all RUNNING containers
    CURRENT_TIME=$(date +%s)
    for ID in $CONTAINER_IDS; do
        # Get service name for logging, fallback to ID
        SERVICE_NAME=$(docker inspect --format '{{ index .Config.Labels "com.docker.compose.service" }}' "$ID" 2>/dev/null)
        if [ -z "$SERVICE_NAME" ]; then
            echo "Warning: Could not find service name for container $ID. Skipping."
            continue
        fi

        # Get CPU usage, remove '%', and get integer part
        CPU_USAGE=$(docker stats --no-stream --format "{{.CPUPerc}}" "$ID" | sed 's/%//' | cut -d'.' -f1)

        # --- Main Logic ---
        if [ "$CPU_USAGE" -ge 99 ]; then
            # CPU is HIGH
            if [[ -v high_cpu_start_times[$ID] ]]; then
                # We are already tracking this container. Check duration.
                START_TIME=${high_cpu_start_times[$ID]}
                DURATION=$((CURRENT_TIME - START_TIME))

                if [ "$DURATION" -ge "$RESTART_AFTER_SECONDS" ]; then
                    echo "FORCE-RESTARTING: $SERVICE_NAME ($ID) at ${CPU_USAGE}% CPU for $DURATION seconds."

                    docker compose kill "$SERVICE_NAME"
                    docker compose start "$SERVICE_NAME"

                    # We unset the timer here, but the container ID will change
                    # after restart. The cleanup logic will handle it.
                    unset high_cpu_start_times[$ID]
                else
                    echo "High CPU: $SERVICE_NAME ($ID) at ${CPU_USAGE}% for $DURATION seconds."
                fi
            else
                # High CPU detected for the first time. Start tracking.
                echo "High CPU detected for $SERVICE_NAME ($ID) at ${CPU_USAGE}%. Starting 5-min timer."
                high_cpu_start_times[$ID]=$CURRENT_TIME
            fi
        else
            # CPU is LOW
            if [[ -v high_cpu_start_times[$ID] ]]; then
                # It was high, but has recovered. Reset the timer.
                echo "CPU normal: $SERVICE_NAME ($ID) at ${CPU_USAGE}%. Resetting timer."
                unset high_cpu_start_times[$ID]
            fi
        fi
    done

    # 2. Cleanup: Remove any tracked containers that are no longer running
    # This also cleans up IDs for containers that were restarted.
    for TRACKED_ID in "${!high_cpu_start_times[@]}"; do
        if [[ ! -v current_running_containers[$TRACKED_ID] ]]; then
            echo "Cleanup: Tracked container $TRACKED_ID is no longer running (likely restarted). Removing from state."
            unset high_cpu_start_times[$TRACKED_ID]
        fi
    done

    sleep "$CHECK_INTERVAL"
done