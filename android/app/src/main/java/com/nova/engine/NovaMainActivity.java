package com.nova.engine;

import android.app.Activity;
import android.os.Bundle;
import android.view.MotionEvent;
import android.view.View;
import android.util.Log;

public class NovaMainActivity extends Activity {
    private static final String TAG = "NOVA_ENGINE";
    private boolean isEngineRunning = false;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        Log.i(TAG, "NovaMainActivity: onCreate - Initializing NOVA 2.0 Engine");
        isEngineRunning = true;
    }

    @Override
    protected void onResume() {
        super.onResume();
        Log.i(TAG, "NovaMainActivity: onResume - Foreground active");
    }

    @Override
    protected void onPause() {
        super.onPause();
        Log.i(TAG, "NovaMainActivity: onPause - Pausing simulation");
    }

    @Override
    protected void onStop() {
        super.onStop();
        Log.i(TAG, "NovaMainActivity: onStop - Backgrounded");
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();
        isEngineRunning = false;
        Log.i(TAG, "NovaMainActivity: onDestroy - Releasing resources");
    }

    @Override
    public boolean onTouchEvent(MotionEvent event) {
        int action = event.getActionMasked();
        float x = event.getX();
        float y = event.getY();

        switch (action) {
            case MotionEvent.ACTION_DOWN:
                Log.d(TAG, "Touch down at: (" + x + ", " + y + ")");
                break;
            case MotionEvent.ACTION_MOVE:
                break;
            case MotionEvent.ACTION_UP:
                Log.d(TAG, "Touch up at: (" + x + ", " + y + ")");
                break;
        }
        return true;
    }
}
