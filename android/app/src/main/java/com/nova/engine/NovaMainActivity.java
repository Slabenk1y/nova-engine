package com.nova.engine;

import android.app.Activity;
import android.content.Context;
import android.graphics.Canvas;
import android.graphics.Color;
import android.graphics.Paint;
import android.opengl.GLES20;
import android.opengl.GLSurfaceView;
import android.opengl.Matrix;
import android.os.Bundle;
import android.util.Log;
import android.view.MotionEvent;
import android.view.View;
import android.widget.FrameLayout;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.FloatBuffer;

import javax.microedition.khronos.egl.EGLConfig;
import javax.microedition.khronos.opengles.GL10;

public class NovaMainActivity extends Activity {
    private static final String TAG = "NOVA_ENGINE";

    static {
        try {
            System.loadLibrary("nova_engine");
            Log.i(TAG, "Successfully loaded native library libnova_engine.so");
        } catch (UnsatisfiedLinkError e) {
            Log.e(TAG, "Could not load libnova_engine.so: " + e.getMessage());
        }
    }

    // Native JNI functions using safe DirectByteBuffers for Zero-Copy & Crash-Free memory access
    public static native void nativeInit();
    public static native void nativeResize(int width, int height);
    public static native void nativeTouchInput(float stickX, float stickY, boolean jumpPressed);
    public static native int nativeStep(float dt, java.nio.ByteBuffer outBuffer, int maxFloats);
    public static native void nativeGetPlayerPos(java.nio.ByteBuffer outPos);

    private GLSurfaceView glSurfaceView;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        Log.i(TAG, "Starting NOVA Obby 3D Activity");

        FrameLayout rootLayout = new FrameLayout(this);

        glSurfaceView = new GLSurfaceView(this);
        glSurfaceView.setEGLContextClientVersion(2);
        glSurfaceView.setRenderer(new NovaGLRenderer());
        rootLayout.addView(glSurfaceView);

        TouchOverlay overlay = new TouchOverlay(this);
        rootLayout.addView(overlay);

        setContentView(rootLayout);
    }

    @Override
    protected void onResume() {
        super.onResume();
        if (glSurfaceView != null) {
            glSurfaceView.onResume();
        }
    }

    @Override
    protected void onPause() {
        super.onPause();
        if (glSurfaceView != null) {
            glSurfaceView.onPause();
        }
    }

    // Real OpenGL ES 2.0 3D Mesh Renderer
    public static class NovaGLRenderer implements GLSurfaceView.Renderer {
        private static final String VERTEX_SHADER =
            "uniform mat4 u_MVPMatrix;\n" +
            "uniform mat4 u_ModelMatrix;\n" +
            "attribute vec4 a_Position;\n" +
            "attribute vec3 a_Normal;\n" +
            "varying vec3 v_Normal;\n" +
            "varying vec3 v_Position;\n" +
            "void main() {\n" +
            "    v_Position = vec3(u_ModelMatrix * a_Position);\n" +
            "    v_Normal = vec3(u_ModelMatrix * vec4(a_Normal, 0.0));\n" +
            "    gl_Position = u_MVPMatrix * a_Position;\n" +
            "}\n";

        private static final String FRAGMENT_SHADER =
            "precision mediump float;\n" +
            "uniform vec4 u_Color;\n" +
            "varying vec3 v_Normal;\n" +
            "varying vec3 v_Position;\n" +
            "void main() {\n" +
            "    vec3 lightPos = vec3(10.0, 30.0, 10.0);\n" +
            "    vec3 lightVector = normalize(lightPos - v_Position);\n" +
            "    float diffuse = max(dot(normalize(v_Normal), lightVector), 0.3);\n" +
            "    gl_FragColor = vec4(u_Color.rgb * diffuse, u_Color.a);\n" +
            "}\n";

        private int program;
        private int mvpMatrixHandle;
        private int modelMatrixHandle;
        private int colorHandle;
        private int positionHandle;
        private int normalHandle;

        private FloatBuffer cubeVertexBuffer;
        private FloatBuffer cubeNormalBuffer;

        private final float[] projMatrix = new float[16];
        private final float[] viewMatrix = new float[16];
        private final float[] modelMatrix = new float[16];
        private final float[] mvpMatrix = new float[16];

        private ByteBuffer playerPosDirectBuffer;
        private FloatBuffer playerPosFloatBuffer;

        private ByteBuffer renderPacketDirectBuffer;
        private FloatBuffer renderPacketFloatBuffer;
        private static final int MAX_OBJECTS = 100;
        private static final int FLOATS_PER_OBJ = 10;

        private long lastFrameTimeNano = System.nanoTime();

        @Override
        public void onSurfaceCreated(GL10 gl, EGLConfig config) {
            GLES20.glEnable(GLES20.GL_DEPTH_TEST);
            GLES20.glDepthFunc(GLES20.GL_LEQUAL);

            initShaders();
            initCubeGeometry();

            playerPosDirectBuffer = ByteBuffer.allocateDirect(3 * 4).order(ByteOrder.nativeOrder());
            playerPosFloatBuffer = playerPosDirectBuffer.asFloatBuffer();

            renderPacketDirectBuffer = ByteBuffer.allocateDirect(MAX_OBJECTS * FLOATS_PER_OBJ * 4)
                .order(ByteOrder.nativeOrder());
            renderPacketFloatBuffer = renderPacketDirectBuffer.asFloatBuffer();

            try {
                NovaMainActivity.nativeInit();
            } catch (UnsatisfiedLinkError e) {
                Log.e(TAG, "nativeInit: " + e.getMessage());
            }
        }

        private void initShaders() {
            int vShader = loadShader(GLES20.GL_VERTEX_SHADER, VERTEX_SHADER);
            int fShader = loadShader(GLES20.GL_FRAGMENT_SHADER, FRAGMENT_SHADER);

            program = GLES20.glCreateProgram();
            GLES20.glAttachShader(program, vShader);
            GLES20.glAttachShader(program, fShader);
            GLES20.glLinkProgram(program);

            mvpMatrixHandle = GLES20.glGetUniformLocation(program, "u_MVPMatrix");
            modelMatrixHandle = GLES20.glGetUniformLocation(program, "u_ModelMatrix");
            colorHandle = GLES20.glGetUniformLocation(program, "u_Color");
            positionHandle = GLES20.glGetAttribLocation(program, "a_Position");
            normalHandle = GLES20.glGetAttribLocation(program, "a_Normal");
        }

        private int loadShader(int type, String shaderCode) {
            int shader = GLES20.glCreateShader(type);
            GLES20.glShaderSource(shader, shaderCode);
            GLES20.glCompileShader(shader);
            return shader;
        }

        private void initCubeGeometry() {
            // Unit cube centered at origin [-1, 1]
            float[] vertices = {
                // Front
                -1f,  1f,  1f,   -1f, -1f,  1f,    1f, -1f,  1f,
                -1f,  1f,  1f,    1f, -1f,  1f,    1f,  1f,  1f,
                // Back
                -1f,  1f, -1f,    1f, -1f, -1f,   -1f, -1f, -1f,
                -1f,  1f, -1f,    1f,  1f, -1f,    1f, -1f, -1f,
                // Top
                -1f,  1f, -1f,   -1f,  1f,  1f,    1f,  1f,  1f,
                -1f,  1f, -1f,    1f,  1f,  1f,    1f,  1f, -1f,
                // Bottom
                -1f, -1f, -1f,    1f, -1f,  1f,   -1f, -1f,  1f,
                -1f, -1f, -1f,    1f, -1f, -1f,    1f, -1f,  1f,
                // Left
                -1f,  1f, -1f,   -1f, -1f, -1f,   -1f, -1f,  1f,
                -1f,  1f, -1f,   -1f, -1f,  1f,   -1f,  1f,  1f,
                // Right
                 1f,  1f,  1f,    1f, -1f,  1f,    1f, -1f, -1f,
                 1f,  1f,  1f,    1f, -1f, -1f,    1f,  1f, -1f,
            };

            float[] normals = {
                 0f,  0f,  1f,   0f,  0f,  1f,   0f,  0f,  1f,   0f,  0f,  1f,   0f,  0f,  1f,   0f,  0f,  1f,
                 0f,  0f, -1f,   0f,  0f, -1f,   0f,  0f, -1f,   0f,  0f, -1f,   0f,  0f, -1f,   0f,  0f, -1f,
                 0f,  1f,  0f,   0f,  1f,  0f,   0f,  1f,  0f,   0f,  1f,  0f,   0f,  1f,  0f,   0f,  1f,  0f,
                 0f, -1f,  0f,   0f, -1f,  0f,   0f, -1f,  0f,   0f, -1f,  0f,   0f, -1f,  0f,   0f, -1f,  0f,
                -1f,  0f,  0f,  -1f,  0f,  0f,  -1f,  0f,  0f,  -1f,  0f,  0f,  -1f,  0f,  0f,  -1f,  0f,  0f,
                 1f,  0f,  0f,   1f,  0f,  0f,   1f,  0f,  0f,   1f,  0f,  0f,   1f,  0f,  0f,   1f,  0f,  0f,
            };

            cubeVertexBuffer = ByteBuffer.allocateDirect(vertices.length * 4)
                .order(ByteOrder.nativeOrder()).asFloatBuffer();
            cubeVertexBuffer.put(vertices).position(0);

            cubeNormalBuffer = ByteBuffer.allocateDirect(normals.length * 4)
                .order(ByteOrder.nativeOrder()).asFloatBuffer();
            cubeNormalBuffer.put(normals).position(0);
        }

        @Override
        public void onSurfaceChanged(GL10 gl, int width, int height) {
            GLES20.glViewport(0, 0, width, height);
            float ratio = (float) width / (float) height;
            Matrix.perspectiveM(projMatrix, 0, 55.0f, ratio, 0.1f, 200.0f);

            try {
                NovaMainActivity.nativeResize(width, height);
            } catch (UnsatisfiedLinkError e) {
                // Handled gracefully
            }
        }

        @Override
        public void onDrawFrame(GL10 gl) {
            long now = System.nanoTime();
            float dt = Math.min((now - lastFrameTimeNano) / 1000000000.0f, 0.05f);
            lastFrameTimeNano = now;

            // Step native Rust physics and retrieve 3D objects via DirectByteBuffer
            int objectCount = 0;
            float pxPlayer = 0f;
            float pyPlayer = 0f;
            float pzPlayer = 0f;

            try {
                objectCount = NovaMainActivity.nativeStep(dt, renderPacketDirectBuffer, MAX_OBJECTS * FLOATS_PER_OBJ);
                NovaMainActivity.nativeGetPlayerPos(playerPosDirectBuffer);

                playerPosFloatBuffer.position(0);
                pxPlayer = playerPosFloatBuffer.get(0);
                pyPlayer = playerPosFloatBuffer.get(1);
                pzPlayer = playerPosFloatBuffer.get(2);
            } catch (UnsatisfiedLinkError e) {
                // Handled gracefully
            }

            // Skybox clear color
            GLES20.glClearColor(0.40f, 0.65f, 0.90f, 1.0f);
            GLES20.glClear(GLES20.GL_COLOR_BUFFER_BIT | GLES20.GL_DEPTH_BUFFER_BIT);

            // Third-person camera tracking behind character
            float camX = pxPlayer;
            float camY = pyPlayer + 4.5f;
            float camZ = pzPlayer + 8.5f;
            Matrix.setLookAtM(viewMatrix, 0, camX, camY, camZ, pxPlayer, pyPlayer + 1.0f, pzPlayer, 0f, 1f, 0f);

            GLES20.glUseProgram(program);

            GLES20.glEnableVertexAttribArray(positionHandle);
            GLES20.glVertexAttribPointer(positionHandle, 3, GLES20.GL_FLOAT, false, 0, cubeVertexBuffer);

            GLES20.glEnableVertexAttribArray(normalHandle);
            GLES20.glVertexAttribPointer(normalHandle, 3, GLES20.GL_FLOAT, false, 0, cubeNormalBuffer);

            // Render all 3D world entities extracted from Rust runtime
            renderPacketFloatBuffer.position(0);
            for (int i = 0; i < objectCount; i++) {
                int base = i * FLOATS_PER_OBJ;
                float px = renderPacketFloatBuffer.get(base + 0);
                float py = renderPacketFloatBuffer.get(base + 1);
                float pz = renderPacketFloatBuffer.get(base + 2);
                float sx = renderPacketFloatBuffer.get(base + 3);
                float sy = renderPacketFloatBuffer.get(base + 4);
                float sz = renderPacketFloatBuffer.get(base + 5);
                float r  = renderPacketFloatBuffer.get(base + 6);
                float g  = renderPacketFloatBuffer.get(base + 7);
                float b  = renderPacketFloatBuffer.get(base + 8);
                float a  = renderPacketFloatBuffer.get(base + 9);

                Matrix.setIdentityM(modelMatrix, 0);
                Matrix.translateM(modelMatrix, 0, px, py, pz);
                Matrix.scaleM(modelMatrix, 0, sx, sy, sz);

                Matrix.multiplyMM(mvpMatrix, 0, viewMatrix, 0, modelMatrix, 0);
                Matrix.multiplyMM(mvpMatrix, 0, projMatrix, 0, mvpMatrix, 0);

                GLES20.glUniformMatrix4fv(mvpMatrixHandle, 1, false, mvpMatrix, 0);
                GLES20.glUniformMatrix4fv(modelMatrixHandle, 1, false, modelMatrix, 0);
                GLES20.glUniform4f(colorHandle, r, g, b, a);

                GLES20.glDrawArrays(GLES20.GL_TRIANGLES, 0, 36);
            }

            GLES20.glDisableVertexAttribArray(positionHandle);
            GLES20.glDisableVertexAttribArray(normalHandle);
        }
    }

    public static class TouchOverlay extends View {
        private final Paint paint = new Paint(Paint.ANTI_ALIAS_FLAG);
        private float joyCenterX = 180f;
        private float joyCenterY = 550f;
        private float joyThumbX = 180f;
        private float joyThumbY = 550f;
        private final float joyRadius = 90f;
        private boolean isJoyActive = false;
        private int joyPointerId = -1;

        private float jumpCenterX = 1050f;
        private float jumpCenterY = 550f;
        private final float jumpRadius = 75f;
        private boolean isJumpPressed = false;
        private int jumpPointerId = -1;

        public TouchOverlay(Context context) {
            super(context);
        }

        @Override
        protected void onSizeChanged(int w, int h, int oldw, int oldh) {
            super.onSizeChanged(w, h, oldw, oldh);
            joyCenterX = w * 0.15f;
            joyCenterY = h * 0.78f;
            joyThumbX = joyCenterX;
            joyThumbY = joyCenterY;

            jumpCenterX = w * 0.88f;
            jumpCenterY = h * 0.78f;
        }

        @Override
        public boolean onTouchEvent(MotionEvent event) {
            int action = event.getActionMasked();
            int actionIndex = event.getActionIndex();
            int pointerId = event.getPointerId(actionIndex);

            if (action == MotionEvent.ACTION_DOWN || action == MotionEvent.ACTION_POINTER_DOWN) {
                float x = event.getX(actionIndex);
                float y = event.getY(actionIndex);

                if (x < getWidth() * 0.5f && !isJoyActive) {
                    isJoyActive = true;
                    joyPointerId = pointerId;
                    joyThumbX = x;
                    joyThumbY = y;
                    sendInput();
                } else if (x >= getWidth() * 0.5f) {
                    float dx = x - jumpCenterX;
                    float dy = y - jumpCenterY;
                    if (Math.sqrt(dx * dx + dy * dy) <= jumpRadius * 1.5f && !isJumpPressed) {
                        isJumpPressed = true;
                        jumpPointerId = pointerId;
                        sendInput();
                    }
                }
                invalidate();
                return true;
            } else if (action == MotionEvent.ACTION_MOVE) {
                for (int i = 0; i < event.getPointerCount(); i++) {
                    int id = event.getPointerId(i);
                    float x = event.getX(i);
                    float y = event.getY(i);

                    if (id == joyPointerId && isJoyActive) {
                        float dx = x - joyCenterX;
                        float dy = y - joyCenterY;
                        float dist = (float) Math.sqrt(dx * dx + dy * dy);
                        if (dist > joyRadius && dist > 0.001f) {
                            joyThumbX = joyCenterX + (dx / dist) * joyRadius;
                            joyThumbY = joyCenterY + (dy / dist) * joyRadius;
                        } else {
                            joyThumbX = x;
                            joyThumbY = y;
                        }
                        sendInput();
                    }
                }
                invalidate();
                return true;
            } else if (action == MotionEvent.ACTION_UP || action == MotionEvent.ACTION_POINTER_UP || action == MotionEvent.ACTION_CANCEL) {
                if (pointerId == joyPointerId) {
                    isJoyActive = false;
                    joyPointerId = -1;
                    joyThumbX = joyCenterX;
                    joyThumbY = joyCenterY;
                    sendInput();
                }
                if (pointerId == jumpPointerId) {
                    isJumpPressed = false;
                    jumpPointerId = -1;
                    sendInput();
                }
                invalidate();
                return true;
            }
            return super.onTouchEvent(event);
        }

        private void sendInput() {
            float stickX = 0f;
            float stickY = 0f;
            if (isJoyActive && joyRadius > 0f) {
                stickX = (joyThumbX - joyCenterX) / joyRadius;
                stickY = -(joyThumbY - joyCenterY) / joyRadius;
            }
            try {
                NovaMainActivity.nativeTouchInput(stickX, stickY, isJumpPressed);
            } catch (UnsatisfiedLinkError e) {
                // Handled gracefully
            }
        }

        @Override
        protected void onDraw(Canvas canvas) {
            super.onDraw(canvas);

            // Header HUD
            paint.setStyle(Paint.Style.FILL);
            paint.setColor(Color.argb(160, 20, 30, 45));
            canvas.drawRect(getWidth() * 0.35f, 20f, getWidth() * 0.65f, 90f, paint);

            paint.setColor(Color.WHITE);
            paint.setTextSize(36f);
            paint.setTextAlign(Paint.Align.CENTER);
            canvas.drawText("NOVA OBBY: RAINBOW TOWER", getWidth() * 0.5f, 65f, paint);

            // Virtual Joystick
            paint.setColor(Color.argb(100, 100, 140, 200));
            paint.setStyle(Paint.Style.FILL);
            canvas.drawCircle(joyCenterX, joyCenterY, joyRadius, paint);

            paint.setStyle(Paint.Style.STROKE);
            paint.setStrokeWidth(5f);
            paint.setColor(Color.argb(180, 255, 255, 255));
            canvas.drawCircle(joyCenterX, joyCenterY, joyRadius, paint);

            paint.setStyle(Paint.Style.FILL);
            paint.setColor(isJoyActive ? Color.argb(220, 60, 150, 255) : Color.argb(150, 180, 180, 200));
            canvas.drawCircle(joyThumbX, joyThumbY, 40f, paint);

            // Jump Button
            paint.setStyle(Paint.Style.FILL);
            paint.setColor(isJumpPressed ? Color.argb(230, 255, 100, 100) : Color.argb(150, 230, 70, 70));
            canvas.drawCircle(jumpCenterX, jumpCenterY, jumpRadius, paint);

            paint.setStyle(Paint.Style.STROKE);
            paint.setStrokeWidth(6f);
            paint.setColor(Color.WHITE);
            canvas.drawCircle(jumpCenterX, jumpCenterY, jumpRadius, paint);

            paint.setStyle(Paint.Style.FILL);
            paint.setTextSize(32f);
            canvas.drawText("JUMP", jumpCenterX, jumpCenterY + 12f, paint);
        }
    }
}
